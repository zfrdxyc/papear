use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Processes {
    pub pids: HashSet<usize>,
    pub names: HashSet<String>,
}

pub fn kill_processes(mut procs: Processes) -> Processes {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    for (pid, proc) in sys.processes() {
        if procs.pids.contains(&(pid.as_u32() as usize))
            && let Some(process) = sys.process(sysinfo::Pid::from(pid.as_u32() as usize))
        {
            process.kill();
            procs.pids.remove(&(pid.as_u32() as usize));
        }
        if procs
            .names
            .contains(proc.name().to_str().unwrap_or_default())
        {
            proc.kill();
            procs.names.remove(proc.name().to_str().unwrap_or_default());
        }
    }
    procs
}
