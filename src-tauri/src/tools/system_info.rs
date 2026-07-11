use super::{Tool, ToolError};
use serde_json::{json, Value};
use sysinfo::{Disks, System};

pub fn tools() -> Vec<Box<dyn Tool>> {
    vec![Box::new(GetSystemInfoTool), Box::new(ListProcessesTool)]
}

pub struct GetSystemInfoTool;

impl Tool for GetSystemInfoTool {
    fn name(&self) -> &'static str {
        "get_system_info"
    }

    fn description(&self) -> &'static str {
        "Get information about the host machine: operating system, kernel version, CPU, total/used RAM, and disk usage."
    }

    fn parameters_schema(&self) -> Value {
        json!({ "type": "object", "properties": {} })
    }

    fn execute(&self, _args: Value) -> Result<Value, ToolError> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_brand = sys
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_default();

        let disks = Disks::new_with_refreshed_list();
        let disk_info: Vec<Value> = disks
            .iter()
            .map(|disk| {
                json!({
                    "mount_point": disk.mount_point().to_string_lossy(),
                    "total_bytes": disk.total_space(),
                    "available_bytes": disk.available_space(),
                })
            })
            .collect();

        Ok(json!({
            "os_name": System::name().unwrap_or_default(),
            "os_version": System::os_version().unwrap_or_default(),
            "kernel_version": System::kernel_version().unwrap_or_default(),
            "host_name": System::host_name().unwrap_or_default(),
            "cpu_brand": cpu_brand,
            "cpu_count": sys.cpus().len(),
            "total_memory_bytes": sys.total_memory(),
            "used_memory_bytes": sys.used_memory(),
            "disks": disk_info,
        }))
    }
}

pub struct ListProcessesTool;

impl Tool for ListProcessesTool {
    fn name(&self) -> &'static str {
        "list_processes"
    }

    fn description(&self) -> &'static str {
        "List running processes on the host machine, sorted by memory or CPU usage (highest first)."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "sort_by": {
                    "type": "string",
                    "enum": ["memory", "cpu"],
                    "description": "Whether to sort by memory usage or CPU usage. Defaults to memory."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of processes to return. Defaults to 10."
                }
            }
        })
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let sort_by = args
            .get("sort_by")
            .and_then(Value::as_str)
            .unwrap_or("memory");
        let limit = args
            .get("limit")
            .and_then(Value::as_u64)
            .unwrap_or(10)
            .clamp(1, 100) as usize;

        let mut sys = System::new_all();
        // Per-process CPU usage needs two samples spaced apart to compute a
        // delta; the very first refresh always reads 0% otherwise.
        if sort_by == "cpu" {
            sys.refresh_all();
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        sys.refresh_all();

        let mut processes: Vec<_> = sys.processes().values().collect();
        match sort_by {
            "cpu" => processes.sort_by(|a, b| {
                b.cpu_usage()
                    .partial_cmp(&a.cpu_usage())
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            _ => processes.sort_by_key(|p| std::cmp::Reverse(p.memory())),
        }

        let top: Vec<Value> = processes
            .into_iter()
            .take(limit)
            .map(|p| {
                json!({
                    "pid": p.pid().as_u32(),
                    "name": p.name().to_string_lossy(),
                    "memory_bytes": p.memory(),
                    "cpu_usage_percent": p.cpu_usage(),
                })
            })
            .collect();

        Ok(json!({ "sorted_by": sort_by, "processes": top }))
    }
}
