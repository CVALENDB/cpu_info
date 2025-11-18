#[cfg(feature = "linux")]
pub mod linux;


#[cfg(feature = "windows")]
pub mod windows;



/// Comprehensive CPU information structure.
///
/// This struct contains all relevant information about the system's CPU,
/// including architecture, manufacturer, model, core counts, and core distribution.
///
/// # Examples
///
/// ```no_run
/// use your_crate::CpuInfo;
///
/// let cpu_info = CpuInfo::new();
/// println!("CPU Model: {}", cpu_info.model);
/// println!("Logical Cores: {:?}", cpu_info.total_logical_cores);
/// println!("Physical Cores: {:?}", cpu_info.total_physical_cores);
/// ```
#[derive(Debug, Clone)]
pub struct CpuInfo {
    /// CPU architecture type (x86, x86_64, ARM, ARM64, etc.)
    pub architecture: CpuArchitecture,
    /// CPU manufacturer/vendor
    pub fabricant: Fabricant,
    /// CPU model name
    pub model: String,
    /// Total number of logical cores (threads)
    pub total_logical_cores: Option<usize>,
    /// Total number of physical cores
    pub total_physical_cores: Option<usize>,
    /// Core distribution information (uniform or hybrid)
    pub distribution: DistributionCore,
}



/// CPU architecture type.
///
/// Represents the instruction set architecture of the CPU.
#[derive(Debug, Clone)]
pub enum CpuArchitecture {
    /// 32-bit x86
    X86,
    /// 64-bit x86 (AMD64/Intel 64)
    X86_64,
    /// 32-bit ARM
    ARM,
    /// 64-bit ARM (AArch64)
    ARM64,
    /// Unknown or unsupported architecture
    Unknown,
}

/// CPU manufacturer/vendor.
///
/// Represents the company that designed or manufactured the CPU.
#[derive(Debug, Clone)]
pub enum Fabricant {
    /// Intel Corporation
    Intel,
    /// Advanced Micro Devices (AMD)
    Amd,
    /// Other manufacturer with vendor string
    Other(String),
    /// Unknown manufacturer
    Unknown,
}

/// Individual CPU core information.
///
/// Contains details about a single logical CPU core (thread).
#[derive(Debug, Clone)]
pub struct Core {
    /// Logical core ID (0-indexed)
    pub id: u32,
    /// Core speed in MHz
    pub speed_mhz: u32,
    /// Physical core ID this logical core belongs to (for hyperthreading detection)
    pub physical_core_id: Option<u32>,
}

impl Core {
    /// Creates a new `Core` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - Logical core ID
    /// * `speed_mhz` - Core speed in MHz
    /// * `physical_core_id` - Physical core ID (None if unavailable)
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::Core;
    ///
    /// let core = Core::new(0, 3600, Some(0));
    /// assert_eq!(core.id, 0);
    /// assert_eq!(core.speed_mhz, 3600);
    /// ```
    pub fn new(id: u32, speed_mhz: u32, physical_core_id: Option<u32>) -> Self {
        Self {
            id,
            speed_mhz,
            physical_core_id,
        }
    }
}

/// CPU core distribution type.
///
/// Describes how CPU cores are organized in terms of frequency:
/// - Traditional CPUs have all cores running at the same frequency (`Lineal`)
/// - Hybrid CPUs have cores at different frequencies (`Hybrid`)
#[derive(Debug, Clone)]
pub enum DistributionCore {
    /// All cores have the same frequency (traditional CPUs).
    ///
    /// # Examples
    ///
    /// - AMD Ryzen 5 5600X (all 6 cores at same speed)
    /// - Intel Core i7-9700K (all 8 cores at same speed)
    Lineal {
        /// Base frequency in MHz
        mhz: u32,
    },
    /// Cores have different frequencies (hybrid architecture).
    ///
    /// # Examples
    ///
    /// - Intel Core i5-12400 (P-cores and E-cores)
    /// - Some ARM big.LITTLE configurations
    /// - AMD CPUs with boost-per-core variations
    Hybrid {
        /// Vector of all cores with individual frequencies
        groups: Vec<Core>,
    },
}