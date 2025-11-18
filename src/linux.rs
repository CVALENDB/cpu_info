use std::fs;
use std::io;
use std::collections::HashSet;
use crate::{Core,CpuArchitecture,CpuInfo,Fabricant,DistributionCore};

#[cfg(feature = "linux")]
impl CpuInfo {
    /// Creates a new `CpuInfo` instance by detecting all CPU information.
    ///
    /// This method performs a single scan of `/sys/devices/system/cpu` to gather
    /// all necessary information efficiently.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use your_crate::CpuInfo;
    ///
    /// let cpu_info = CpuInfo::new();
    /// match cpu_info.distribution {
    ///     DistributionCore::Lineal { mhz } => {
    ///         println!("Uniform CPU with all cores at {} MHz", mhz);
    ///     }
    ///     DistributionCore::Hybrid { ref groups } => {
    ///         println!("Hybrid CPU with {} cores at different speeds", groups.len());
    ///     }
    /// }
    /// ```
    pub fn new() -> Self {
        // Count ALL cores first (independent of cpufreq availability)
        let total_logical_cores = Self::get_total_logical_cores();
        
        // Get detailed core information (may be partial on some systems)
        let cores = Self::get_cores();

        // Derive physical cores from detailed info or use fallback
        let total_physical_cores = if !cores.is_empty() {
            let mut physical_ids = HashSet::new();
            for core in &cores {
                if let Some(id) = core.physical_core_id {
                    physical_ids.insert(id);
                }
            }
            if physical_ids.is_empty() {
                Self::get_total_physical_cores_fallback()
            } else {
                Some(physical_ids.len())
            }
        } else {
            Self::get_total_physical_cores_fallback()
        };

        let distribution = Self::detect_distribution(&cores);

        Self {
            architecture: Self::get_architecture(),
            fabricant: Self::get_fabricant().unwrap_or(Fabricant::Unknown),
            model: Self::get_model().unwrap_or("Unknown".to_string()),
            total_logical_cores,
            total_physical_cores,
            distribution,
        }
    }

    /// Detects the CPU architecture using Rust's built-in constants.
    ///
    /// This method is compile-time safe and doesn't require any system calls.
    fn get_architecture() -> CpuArchitecture {
        match std::env::consts::ARCH {
            "x86_64" => CpuArchitecture::X86_64,
            "aarch64" => CpuArchitecture::ARM64,
            "arm" => CpuArchitecture::ARM,
            "x86" => CpuArchitecture::X86,
            _ => CpuArchitecture::Unknown,
        }
    }

    /// Detects the CPU manufacturer/vendor.
    ///
    /// On x86/x86_64, this uses the CPUID instruction for language-independent detection.
    /// On ARM and other architectures, it reads from `/proc/cpuinfo`.
    fn get_fabricant() -> Result<Fabricant, io::Error> {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            return Self::get_fabricant_cpuid();
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            Self::get_fabricant_arm()
        }
    }

    /// Uses CPUID instruction to detect CPU vendor on x86/x86_64.
    ///
    /// This method is language-independent and works regardless of system locale.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn get_fabricant_cpuid() -> Result<Fabricant, io::Error> {
        #[cfg(target_arch = "x86")]
        use core::arch::x86::__cpuid;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::__cpuid;

        unsafe {
            let result = __cpuid(0);
            
            // EBX, EDX, ECX contain the vendor string (12 bytes)
            let mut vendor = [0u8; 12];
            vendor[0..4].copy_from_slice(&result.ebx.to_le_bytes());
            vendor[4..8].copy_from_slice(&result.edx.to_le_bytes());
            vendor[8..12].copy_from_slice(&result.ecx.to_le_bytes());
            
            Ok(match &vendor {
                b"GenuineIntel" => Fabricant::Intel,
                b"AuthenticAMD" => Fabricant::Amd,
                _ => Fabricant::Other(String::from_utf8_lossy(&vendor).trim().to_string()),
            })
        }
    }

    /// Detects CPU manufacturer on ARM by reading the implementer ID.
    ///
    /// This method parses hexadecimal implementer IDs and maps them to known vendors.
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    fn get_fabricant_arm() -> Result<Fabricant, io::Error> {
        let content = fs::read_to_string("/proc/cpuinfo")?;
        
        for line in content.lines() {
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_lowercase();
                if key.contains("implementer") {
                    let value = line[colon_pos + 1..].trim();
                    
                    // Parse the value (can be "0x41" or "41")
                    let implementer = if let Some(hex_str) = value.strip_prefix("0x") {
                        u32::from_str_radix(hex_str, 16).ok()
                    } else {
                        u32::from_str_radix(value, 16).ok()
                    };
                    
                    return Ok(match implementer {
                        Some(0x41) => Fabricant::Other("ARM".to_string()),
                        Some(0x42) => Fabricant::Other("Broadcom".to_string()),
                        Some(0x43) => Fabricant::Other("Cavium".to_string()),
                        Some(0x44) => Fabricant::Other("DEC".to_string()),
                        Some(0x4e) => Fabricant::Other("Nvidia".to_string()),
                        Some(0x50) => Fabricant::Other("APM".to_string()),
                        Some(0x51) => Fabricant::Qualcomm,
                        Some(0x56) => Fabricant::Other("Marvell".to_string()),
                        Some(0x61) => Fabricant::Other("Apple".to_string()),
                        _ => Fabricant::Other(value.to_string()),
                    });
                }
            }
        }
        
        Ok(Fabricant::Unknown)
    }

    /// Detects the CPU model name.
    ///
    /// On x86/x86_64, this uses CPUID for reliable detection.
    /// Falls back to reading `/proc/cpuinfo` if CPUID is unavailable.
    fn get_model() -> Result<String, io::Error> {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if let Ok(model) = Self::get_model_cpuid() {
                return Ok(model);
            }
        }

        Self::get_model_procfs()
    }

    /// Uses CPUID extended functions to get the CPU brand string on x86/x86_64.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn get_model_cpuid() -> Result<String, io::Error> {
        #[cfg(target_arch = "x86")]
        use core::arch::x86::__cpuid;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::__cpuid;

        unsafe {
            let ext_result = __cpuid(0x80000000);
            if ext_result.eax < 0x80000004 {
                return Err(io::Error::new(io::ErrorKind::NotFound, "Extended CPUID not supported"));
            }

            let mut brand = [0u8; 48];
            
            // Read the 3 registers containing the brand string
            for i in 0..3 {
                let result = __cpuid(0x80000002 + i);
                let offset = i as usize * 16;
                brand[offset..offset + 4].copy_from_slice(&result.eax.to_le_bytes());
                brand[offset + 4..offset + 8].copy_from_slice(&result.ebx.to_le_bytes());
                brand[offset + 8..offset + 12].copy_from_slice(&result.ecx.to_le_bytes());
                brand[offset + 12..offset + 16].copy_from_slice(&result.edx.to_le_bytes());
            }
            
            let model = String::from_utf8_lossy(&brand).trim().to_string();
            
            if model.is_empty() {
                Err(io::Error::new(io::ErrorKind::NotFound, "Model not found"))
            } else {
                Ok(model)
            }
        }
    }

    /// Reads the CPU model name from `/proc/cpuinfo`.
    ///
    /// This method uses case-insensitive comparison to handle different locales.
    fn get_model_procfs() -> Result<String, io::Error> {
        let content = fs::read_to_string("/proc/cpuinfo")?;
        
        for line in content.lines() {
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim();
                // Search for "model name" case-insensitive
                if key.eq_ignore_ascii_case("model name") {
                    return Ok(line[colon_pos + 1..].trim().to_string());
                }
            }
        }
        
        Err(io::Error::new(io::ErrorKind::NotFound, "Model not found"))
    }

    /// Counts all logical CPU cores by scanning `/sys/devices/system/cpu`.
    ///
    /// This method counts all `cpuN` directories regardless of cpufreq availability.
    fn get_total_logical_cores() -> Option<usize> {
        let count = fs::read_dir("/sys/devices/system/cpu")
            .ok()?
            .flatten()
            .filter(|entry| {
                entry.file_name()
                    .to_str()
                    .and_then(|s| s.strip_prefix("cpu"))
                    .map_or(false, |rest| rest.parse::<u32>().is_ok())
            })
            .count();

        if count > 0 {
            Some(count)
        } else {
            None
        }
    }

    /// Counts physical cores by reading topology information from sysfs.
    ///
    /// This is used as a fallback when detailed core information is unavailable.
    fn get_total_physical_cores_fallback() -> Option<usize> {
        let mut core_ids = HashSet::new();
        
        let entries = fs::read_dir("/sys/devices/system/cpu").ok()?;
        
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_str()?;
            
            if let Some(rest) = name_str.strip_prefix("cpu") {
                if rest.parse::<u32>().is_ok() {
                    let core_id_path = entry.path().join("topology/core_id");
                    if let Ok(core_id_str) = fs::read_to_string(core_id_path) {
                        if let Ok(core_id) = core_id_str.trim().parse::<u32>() {
                            core_ids.insert(core_id);
                        }
                    }
                }
            }
        }
        
        if core_ids.is_empty() {
            None
        } else {
            Some(core_ids.len())
        }
    }

    /// Detects CPU core distribution by analyzing core frequencies.
    ///
    /// Returns `Lineal` if all cores have the same frequency (traditional CPUs),
    /// or `Hybrid` if cores have different frequencies (e.g., Intel 12th gen+, some ARM).
    fn detect_distribution(cores: &[Core]) -> DistributionCore {
        // If we have no core information, return Lineal with 0 MHz
        if cores.is_empty() || cores.iter().all(|c| c.speed_mhz == 0) {
            return DistributionCore::Lineal { mhz: 0 };
        }

        // Frequency-based detection
        let mut cores = cores.to_vec();
        cores.sort_by_key(|c| c.speed_mhz);

        let all_same = cores.windows(2).all(|w| w[0].speed_mhz == w[1].speed_mhz);
        
        if all_same {
            return DistributionCore::Lineal { 
                mhz: cores[0].speed_mhz 
            };
        }

        // If not uniform, store all cores with their individual frequencies
        DistributionCore::Hybrid { 
            groups: cores 
        }
    }

    /// Reads detailed information for all CPU cores.
    ///
    /// This method attempts to read frequency and topology information for each core.
    /// Cores are included even if frequency information is unavailable (speed_mhz = 0),
    /// which is useful for accurate physical core counting.
    fn get_cores() -> Vec<Core> {
        let mut cores = Vec::new();

        let Ok(entries) = fs::read_dir("/sys/devices/system/cpu") else {
            return cores;
        };

        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = match name.to_str() {
                Some(s) if s.starts_with("cpu") => s,
                _ => continue,
            };

            let id: u32 = match name[3..].parse() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let cpu_path = entry.path();

            // Try to read frequency (may not exist on some systems)
            let speed_khz = fs::read_to_string(cpu_path.join("cpufreq/cpuinfo_max_freq"))
                .or_else(|_| fs::read_to_string(cpu_path.join("cpufreq/scaling_max_freq")))
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok())
                .unwrap_or(0);

            // Read physical core ID (should always exist)
            let physical_core_id = fs::read_to_string(cpu_path.join("topology/core_id"))
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok());

            // Include the core even if speed_khz is 0
            // (useful for accurate physical core counting)
            cores.push(Core {
                id,
                speed_mhz: speed_khz / 1000,
                physical_core_id,
            });
        }

        cores
    }
}