#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use embassy_executor::_export::StaticCell;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Config, Ipv4Address, Stack, StackResources};

use embassy_executor::Executor;
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_sync::channel::Receiver;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_backtrace as _;
use esp_hal_smartled::{smartLedAdapter, SmartLedsAdapter};
use esp_println::logger::init_logger;
use esp_println::println;
use esp_wifi::initialize;
use esp_wifi::wifi::{WifiController, WifiDevice, WifiEvent, WifiMode, WifiState};
use hal::clock::{ClockControl, CpuClock};
use hal::cpu_control::CpuControl;
use hal::gpio::{
    Bank1GpioRegisterAccess, DualCoreInteruptStatusRegisterAccessBank1, Gpio33Signals, GpioPin,
    InputOutputAnalogPinType,
};
use hal::pulse_control::ConfiguredChannel0;
use hal::{embassy, peripherals::Peripherals, prelude::*, timer::TimerGroup, Rtc, IO};
use hal::{PulseControl, Rng};
use smart_leds::SmartLedsWrite;
use smart_leds::RGB8;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        let (x,) = STATIC_CELL.init(($val,));
        x
    }};
}
#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;

    extern "C" {
        static mut _heap_start: u32;
    }
    unsafe {
        let heap_start = &_heap_start as *const _ as usize;
        ALLOCATOR.init(heap_start as *mut u8, HEAP_SIZE);
    }
}
static EXECUTOR: StaticCell<Executor> = StaticCell::new();
#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Info);

    init_heap();

    let peripherals = Peripherals::take();

    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    rtc.rwdt.disable();

    let timer = TimerGroup::new(peripherals.TIMG1, &clocks).timer0;
    initialize(
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    let (wifi, _) = peripherals.RADIO.split();
    let (wifi_interface, controller) = esp_wifi::wifi::new_with_mode(wifi, WifiMode::Sta);

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0.timer0);

    let config = Config::Dhcp(Default::default());

    let seed = 123456; // very random, very secure seed

    // Init network stack
    let stack = &*singleton!(Stack::new(
        wifi_interface,
        config,
        singleton!(StackResources::<3>::new()),
        seed
    ));

    let executor = EXECUTOR.init(Executor::new());
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let pulse = PulseControl::new(peripherals.RMT, &mut system.peripheral_clock_control).unwrap();
    let mut led = <smartLedAdapter!(23)>::new(pulse.channel0, io.pins.gpio33);
    led.write([RGB8::default(); 23].into_iter()).unwrap();
    let channel: Channel<Mutex>> = embassy_sync::channel::Channel::new();
    let sender = channel.sender();
    let receiver = channel.receiver();
    executor.run(|spawner| {
        spawner.spawn(connection(controller)).ok();
        spawner.spawn(net_task(&stack)).ok();
        spawner.spawn(task(&stack)).ok();
        spawner.spawn(led_task(led)).ok();
    });
}
#[embassy_executor::task]
async fn led_task(
    mut leds: SmartLedsAdapter<
        ConfiguredChannel0<
            'static,
            GpioPin<
                hal::gpio::Unknown,
                Bank1GpioRegisterAccess,
                DualCoreInteruptStatusRegisterAccessBank1,
                InputOutputAnalogPinType,
                Gpio33Signals,
                33,
            >,
        >,
        553,
    >,
) {
    loop {
        leds.write([RGB8::new(0, 0, 255); 23].into_iter()).unwrap();
        Timer::after(Duration::from_millis(1000)).await;
        leds.write([RGB8::new(0, 255, 0); 23].into_iter()).unwrap();
        Timer::after(Duration::from_millis(1000)).await;
        leds.write([RGB8::new(255, 0, 0); 23].into_iter()).unwrap();
        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    println!("start connection task");
    println!("Device capabilities: {:?}", controller.get_capabilities());
    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.into(),
                password: PASSWORD.into(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            println!("Starting wifi");
            controller.start().await.unwrap();
            println!("Wifi started!");
        }
        println!("About to connect...");

        match controller.connect().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static>>) {
    stack.run().await
}

#[embassy_executor::task]
async fn task(stack: &'static Stack<WifiDevice<'static>>) {
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    println!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config() {
            println!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}
