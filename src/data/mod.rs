use std::{ffi::OsString, path::PathBuf};
use sysinfo::{DiskKind, IpNetwork, MacAddr, Pid, System, Users};

#[derive(Clone, Debug)]
pub struct Data {
    pub cpu: CpuInfo,
    pub processes: Vec<Process>,
    pub disks: Vec<Disk>,
    pub memory: Memory,
    pub networks: Vec<Network>,
}

#[derive(Default, Clone, Debug)]
pub struct Memory {
    pub used_swap: u64,
    pub total_swap: u64,
    pub used_mem: u64,
    pub total_mem: u64,
}

#[derive(Default, Clone, Debug)]
pub struct Core {
    pub name: String,
    pub usage: f32,
    pub frequency: u64,
}

impl From<&sysinfo::Cpu> for Core {
    fn from(value: &sysinfo::Cpu) -> Self {
        Self {
            name: value.name().to_string(),
            usage: value.cpu_usage(),
            frequency: value.frequency(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Disk {
    pub name: String,
    pub mount_point: PathBuf,
    pub kind: DiskKind,
    pub total_space: u64,
    pub free_space: u64,
    pub read_only: bool,
    pub removable: bool,
}

impl From<&sysinfo::Disk> for Disk {
    fn from(value: &sysinfo::Disk) -> Self {
        Self {
            name: value.name().to_str().unwrap().to_string(),
            mount_point: value.mount_point().to_path_buf(),
            kind: value.kind(),
            total_space: value.total_space(),
            free_space: value.available_space(),
            read_only: value.is_read_only(),
            removable: value.is_removable(),
        }
    }
}

impl Disk {
    fn get_vec_from_sysinfo() -> Vec<Self> {
        sysinfo::Disks::new_with_refreshed_list()
            .into_iter()
            .map(|x| x.into())
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Network {
    pub name: String,
    pub ip_addresses: Vec<IpNetwork>,
    pub mac_address: MacAddr,
    pub received: u64,
    pub transmitted: u64,
}

impl From<(&String, &sysinfo::NetworkData)> for Network {
    fn from(value: (&String, &sysinfo::NetworkData)) -> Self {
        let mut ips: Vec<IpNetwork> = value
            .1
            .ip_networks()
            .into_iter()
            .map(|x| x.clone())
            .collect();
        ips.sort_by(|a, b| a.cmp(b));
        Self {
            name: value.0.to_string(),
            ip_addresses: ips,
            mac_address: value.1.mac_address(),
            received: value.1.received(),
            transmitted: value.1.transmitted(),
        }
    }
}

impl Network {
    fn get_vec_from_sysinfo() -> Vec<Self> {
        let networks = sysinfo::Networks::new_with_refreshed_list();

        let mut networks: Vec<Network> = networks.into_iter().map(|x| x.into()).collect();

        networks.sort_by(|a, b| a.name.cmp(&b.name));

        networks
    }
}

#[derive(Default, Clone, Debug)]
pub struct CpuInfo {
    pub vendor: String,
    pub brand: String,
    pub usage: f32,
    pub cores: Vec<Core>,
}

#[derive(Clone, Debug)]
pub struct Process {
    pub user: String,
    pub name: OsString,
    pub pid: Pid,
    pub memory: u64,
    pub cpu: f32,
    pub command: String,
    pub run_time: u64,
    pub total_m: u64,
}

impl Data {
    pub fn new(sys: &mut System, users: &Users) -> Self {
        let cpu = sys.cpus();

        let cpu = CpuInfo {
            brand: cpu[0].brand().to_string(),
            vendor: cpu[0].vendor_id().to_string(),
            usage: sys.global_cpu_usage(),
            cores: cpu.iter().map(|x| x.into()).collect::<Vec<Core>>(),
        };

        let memory = Memory {
            total_mem: sys.total_memory(),
            total_swap: sys.total_swap(),
            used_mem: sys.used_memory(),
            used_swap: sys.used_swap(),
        };

        let mut processes = sys
            .processes()
            .iter()
            .map(|(p, x)| Process {
                pid: *p,
                name: x.name().to_owned(),
                user: if let Some(x) = x.user_id() {
                    users.get_user_by_id(x).unwrap().name().to_string()
                } else {
                    String::new()
                },
                command: match x.cmd().first() {
                    Some(x) => x.to_str().unwrap().to_string(),
                    None => x.name().to_str().unwrap().to_string(),
                },
                cpu: x.cpu_usage() / cpu.cores.len() as f32,
                memory: x.memory(),
                run_time: x.run_time(),
                total_m: memory.total_mem,
            })
            .collect::<Vec<Process>>();
        processes.sort_by(|a, b| b.cpu.total_cmp(&a.cpu));

        let disks = Disk::get_vec_from_sysinfo();
        let networks = Network::get_vec_from_sysinfo();

        Self {
            cpu,
            processes,
            disks,
            memory,
            networks,
        }
    }
}
