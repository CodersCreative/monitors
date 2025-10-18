use crate::{
    cores::{Core, CpuInfo},
    processes::Process,
};
use sysinfo::{Components, Cpu, Disks, LoadAvg, Networks, System, Users};

pub struct Data {
    pub cpu: CpuInfo,
    pub processes: Vec<Process>,
    pub disks: Disks,
    pub memory: Memory,
    pub networks: Networks,
    pub components: Components,
}

pub struct Memory {
    pub used_swap: u64,
    pub total_swap: u64,
    pub used_mem: u64,
    pub total_mem: u64,
}

impl Data {
    pub fn new(sys: &mut System, users: &Users) -> Self {
        let cpu = sys.cpus();
        let cpu = CpuInfo {
            brand: cpu[0].brand().to_string(),
            vendor: cpu[0].vendor_id().to_string(),
            usage: sys.global_cpu_usage(),
            cores: cpu
                .iter()
                .map(|x| Core {
                    name: x.name().to_string(),
                    usage: x.cpu_usage(),
                    frequency: x.frequency(),
                })
                .collect::<Vec<Core>>(),
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
                user: users
                    .get_user_by_id(x.user_id().unwrap())
                    .unwrap()
                    .name()
                    .to_string(),
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

        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();

        Self {
            cpu,
            processes,
            disks,
            memory,
            networks,
            components,
        }
    }
}
