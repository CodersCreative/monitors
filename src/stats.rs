use std::net::IpAddr;

use hw_linux::InfoTrait;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Gauge, List, ListItem, Row, Table, Widget},
    Frame,
};
use sysinfo::{Disk, Disks, IpNetwork, NetworkData, Networks};

use crate::{
    cores::CpuInfo,
    data::{Data, Memory},
    get_time,
    packages::PackageManagers,
};

const Y: usize = 6;
const X_1: usize = 1;
const X_2: usize = 2;

pub fn draw_page_1(frame: &mut Frame, area: Rect, data: &Data, pms: &PackageManagers) {
    let is_linux = hw_linux::is_linux().unwrap_or(false);
    let vertical = Layout::vertical([Constraint::Ratio(1, Y as u32); Y]);
    let horizontal = Layout::horizontal([Constraint::Ratio(1, X_1 as u32); X_1]);
    let v = vertical.areas::<Y>(area);
    let mut areas = Vec::new();
    for a in v {
        for l in horizontal.areas::<X_1>(a) {
            areas.push(l);
        }
    }

    let mut widgets = Vec::new();
    if is_linux {
        widgets.push(host());
        widgets.push(kernel());
        widgets.push(environment());
    }

    widgets.push(packages(pms));
    widgets.push(cpu(&data.cpu));
    widgets.push(memory(&data.memory));

    for (i, widget) in widgets
        .into_iter()
        .enumerate()
        .filter(|(i, _)| i < &areas.len())
    {
        frame.render_widget(widget, areas[i]);
    }
}

pub fn draw_page_2(frame: &mut Frame, area: Rect, data: &Data) {
    let is_linux = hw_linux::is_linux().unwrap_or(false);
    let vertical = Layout::vertical([Constraint::Ratio(1, Y as u32); Y]);
    let horizontal = Layout::horizontal([Constraint::Ratio(1, X_2 as u32); X_2]);
    let v = vertical.areas::<Y>(area);
    let mut areas = Vec::new();
    for a in v {
        for l in horizontal.areas::<X_2>(a) {
            areas.push(l);
        }
    }

    let mut widgets = Vec::new();

    if is_linux {
        let gpus = hw_linux::gpu::Gpus::get().unwrap();
        widgets.append(&mut gpus.0.iter().enumerate().map(|(i, x)| gpu(x, i)).collect());
    }

    widgets.append(&mut data.disks.iter().map(|x| disk(x)).collect());
    let mut networks: Vec<(&String, &NetworkData)> = data.networks.iter().collect();
    networks.sort_by(|a, b| a.0.cmp(b.0));

    widgets.append(&mut networks.into_iter().map(|x| network(x.0, x.1)).collect());

    for (i, widget) in widgets
        .into_iter()
        .enumerate()
        .filter(|(i, _)| i < &areas.len())
    {
        frame.render_widget(widget, areas[i]);
    }
}

fn memory(memory: &Memory) -> Table<'static> {
    let rows = vec![
        Row::new(vec![
            "Total Mem".to_string(),
            format!(
                "{:.2} Gb",
                (memory.total_mem as f64 / 1024_f64 / 1024_f64 / 1024_f64)
            ),
        ]),
        Row::new(vec![
            "Free Mem".to_string(),
            format!(
                "{:.2} Gb",
                ((memory.total_mem - memory.used_mem) as f64 / 1024_f64 / 1024_f64 / 1024_f64)
            ),
        ]),
        Row::new(vec![
            "Used Mem".to_string(),
            format!(
                "{:.2} Gb",
                (memory.used_mem as f64 / 1024_f64 / 1024_f64 / 1024_f64)
            ),
        ]),
        Row::new(vec![
            "Total Swap".to_string(),
            format!(
                "{:.2} Gb",
                (memory.total_swap as f64 / 1024_f64 / 1024_f64 / 1024_f64)
            ),
        ]),
        Row::new(vec![
            "Free Swap".to_string(),
            format!(
                "{:.2} Gb",
                ((memory.total_swap - memory.used_swap) as f64 / 1024_f64 / 1024_f64 / 1024_f64)
            ),
        ]),
    ];

    let widths = [
        Constraint::Percentage(25),
        Constraint::Fill(1),
        Constraint::Percentage(25),
        Constraint::Fill(1),
    ];
    let block = get_block().title("Memory");

    // Gauge::default()
    //     .percent((memory.used_mem / memory.total_mem * 100) as u16)
    //     .label(format!("{:.2}%", (memory.used_mem / memory.total_mem)))
    //     .block(block)
    //     .gauge_style(Style::default().fg(Color::Rgb(137, 180, 250)));

    Table::new(rows, widths).block(block)
}

fn packages(pms: &PackageManagers) -> Table<'static> {
    let rows = pms
        .0
        .windows(2)
        .step_by(2)
        .map(|pm| {
            Row::new(vec![
                pm[0].name.clone(),
                format!("{}", pm[0].packages),
                pm[1].name.clone(),
                format!("{}", pm[1].packages),
            ])
        })
        .collect::<Vec<Row>>();

    let widths = [
        Constraint::Percentage(25),
        Constraint::Fill(1),
        Constraint::Percentage(25),
        Constraint::Fill(1),
    ];
    let block = get_block().title("Packages");
    Table::new(rows, widths).block(block)
}

fn environment() -> Table<'static> {
    let env_info = hw_linux::environment::EnvironmentInfo::get().unwrap();
    let mut rows = Vec::new();
    if let Some(user) = env_info.user {
        rows.push(Row::new(vec!["User".to_string(), user.to_string()]));
    }
    if let Some(shell) = env_info.shell {
        rows.push(Row::new(vec!["Shell".to_string(), shell.to_string()]));
    }
    if let Some(term) = env_info.term {
        rows.push(Row::new(vec!["Terminal".to_string(), term.to_string()]));
    }
    let widths = [Constraint::Percentage(20), Constraint::Fill(1)];
    let block = get_block().title("Environment");
    Table::new(rows, widths).block(block)
}

fn gpu(gpu: &hw_linux::gpu::Gpu, index: usize) -> Table<'static> {
    let mut rows = Vec::new();
    if let Some(name) = &gpu.name {
        rows.push(Row::new(vec!["Name".to_string(), name.to_string()]));
    }
    if let Some(vendor) = &gpu.vendor {
        rows.push(Row::new(vec!["Vendor".to_string(), vendor.to_string()]));
    }
    if let Some(driver) = &gpu.driver {
        rows.push(Row::new(vec!["Driver".to_string(), driver.to_string()]));
    }
    let widths = [Constraint::Percentage(20), Constraint::Fill(1)];
    let block = get_block().title(format!("GPU {}", index + 1));
    Table::new(rows, widths).block(block)
}

fn disk(disk: &Disk) -> Table<'static> {
    let mut rows = Vec::new();
    rows.push(Row::new(vec![
        "Mount".to_string(),
        format!("{:?}", disk.mount_point()),
    ]));

    rows.push(Row::new(vec![
        "Kind".to_string(),
        format!("{}", disk.kind()),
    ]));

    rows.push(Row::new(vec![
        "Total".to_string(),
        format!("{:.2} Gb", disk.total_space() / 1024 / 1024 / 1024),
    ]));

    rows.push(Row::new(vec![
        "Free".to_string(),
        format!("{:.2} Gb", disk.available_space() / 1024 / 1024 / 1024),
    ]));

    rows.push(Row::new(vec![
        "Read Only?".to_string(),
        match disk.is_read_only() {
            true => "Yes".to_string(),
            false => "No".to_string(),
        },
    ]));

    rows.push(Row::new(vec![
        "Removable?".to_string(),
        match disk.is_removable() {
            true => "Yes".to_string(),
            false => "No".to_string(),
        },
    ]));

    let widths = [Constraint::Percentage(30), Constraint::Fill(1)];
    let block = get_block().title(format!("Disk {:?}", disk.name()));
    Table::new(rows, widths).block(block)
}

fn host() -> Table<'static> {
    let host_info = hw_linux::host::HostInfo::get().unwrap();
    let mut rows = Vec::new();
    if let Some(model) = host_info.model {
        rows.push(Row::new(vec!["Device".to_string(), model.to_string()]));
    }
    if let Some(arch) = host_info.architecture {
        rows.push(Row::new(vec!["Arch".to_string(), arch.to_string()]));
    }
    if let Some(vendor) = host_info.vendor {
        rows.push(Row::new(vec!["Vendor".to_string(), vendor.to_string()]));
    }
    if let Some(os) = host_info.os {
        rows.push(Row::new(vec!["OS".to_string(), os.to_string()]));
    }
    if let Some(distro) = host_info.distro {
        rows.push(Row::new(vec!["Distro".to_string(), distro.to_string()]));
    }
    if let Some(de) = host_info.desktop_env {
        rows.push(Row::new(vec!["DE".to_string(), de.to_string()]));
    }
    if let Some(wm) = host_info.win_manager {
        rows.push(Row::new(vec!["WM".to_string(), wm.to_string()]));
    }
    if let Some(session) = host_info.session {
        rows.push(Row::new(vec!["Session".to_string(), session.to_string()]));
    }

    let widths = [Constraint::Percentage(30), Constraint::Fill(1)];
    let block = get_block().title("Host");
    Table::new(rows, widths).block(block)
}

fn kernel() -> Table<'static> {
    let kernel_info = hw_linux::environment::KernelInfo::get().unwrap();
    let uptime_info = hw_linux::environment::UptimeInfo::get().unwrap_or_default();

    let mut rows = Vec::new();
    if let Some(version) = kernel_info.version {
        rows.push(Row::new(vec!["Version".to_string(), version.to_string()]));
    }
    if let Some(release) = kernel_info.release {
        rows.push(Row::new(vec!["Release".to_string(), release.to_string()]));
    }
    if let Some(uptime_s) = uptime_info.0 {
        rows.push(Row::new(vec![
            "Uptime".to_string(),
            get_time(uptime_s as u64),
        ]));
    }
    let widths = [Constraint::Percentage(30), Constraint::Fill(1)];
    let block = get_block().title("Kernel");
    Table::new(rows, widths).block(block)
}

fn network(txt: &str, network: &NetworkData) -> Table<'static> {
    let mut rows = Vec::new();

    let mut ips: Vec<&IpNetwork> = network.ip_networks().iter().collect();
    ips.sort_by(|a, b| a.cmp(b));

    for (i, ip) in ips.iter().enumerate() {
        rows.push(Row::new(vec![
            format!("Ip {i}"),
            format!("{}/{}", ip.addr.to_string(), ip.prefix),
        ]));
    }

    rows.push(Row::new(vec![
        "Mac".to_string(),
        network.mac_address().to_string(),
    ]));

    rows.push(Row::new(vec![
        "Received".to_string(),
        network.received().to_string(),
    ]));

    rows.push(Row::new(vec![
        "Transmitted".to_string(),
        network.transmitted().to_string(),
    ]));

    let widths = [Constraint::Percentage(30), Constraint::Fill(1)];
    let block = get_block().title(format!("Network {:?}", txt));
    Table::new(rows, widths).block(block)
}

fn cpu(cpu_info: &CpuInfo) -> Table<'static> {
    let linux_cpu_info = hw_linux::cpu::CpuInfo::get().unwrap_or_default();

    let mut rows = Vec::new();

    rows.push(Row::new(vec![
        "Name".to_string(),
        cpu_info.brand.to_string(),
    ]));

    rows.push(Row::new(vec![
        "Vendor".to_string(),
        cpu_info.vendor.to_string(),
    ]));

    if let (Some(cores), Some(threads)) = (linux_cpu_info.cores, linux_cpu_info.threads) {
        rows.push(Row::new(vec![
            "Cores".to_string(),
            format!("{} ({})", cores, threads),
        ]));
    } else {
        rows.push(Row::new(vec![
            "Cores".to_string(),
            format!("{} ", cpu_info.cores.len()),
        ]));
    }

    if let Some(cache) = linux_cpu_info.cache {
        rows.push(Row::new(vec![
            "Cache".to_string(),
            format!("{:.2} kb", (cache / 1024_f64)),
        ]));
    }

    if let (Some(freq), Some(max)) = (linux_cpu_info.cur_freq, linux_cpu_info.max_freq) {
        rows.push(Row::new(vec![
            "Frequency".to_string(),
            format!("{:.2} MHz / {:.0} MHz", freq, max),
        ]));
    } else {
        rows.push(Row::new(vec![
            "Frequency".to_string(),
            format!("{:.2} MHz", cpu_info.cores[0].frequency),
        ]));
    }

    if let Some(temp) = linux_cpu_info.temp {
        rows.push(Row::new(vec![
            "Temp".to_string(),
            format!("{:.2} °C", temp),
        ]));
    }

    let widths = [Constraint::Percentage(30), Constraint::Fill(1)];
    let block = get_block().title("CPU");
    Table::new(rows, widths).block(block)
}

pub fn get_block() -> Block<'static> {
    Block::bordered()
        .title_alignment(Alignment::Center)
        .title_style(Style::default().fg(Color::Red))
        .border_type(BorderType::Rounded)
}
