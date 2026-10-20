//! `DebugEntrySystemSpecs`: the runtime, the processor, the display and the graphics device.

use super::super::layout::Line;

/// `DeviceType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Other,
    Integrated,
    Discrete,
    Virtual,
    Cpu,
}

impl DeviceType {
    /// `VulkanPhysicalDevice.deviceType`: from `VkPhysicalDeviceType`'s value.
    pub fn from_vulkan(raw: i32) -> Self {
        match raw {
            1 => Self::Integrated,
            2 => Self::Discrete,
            3 => Self::Virtual,
            4 => Self::Cpu,
            _ => Self::Other,
        }
    }

    fn suffix(self) -> &'static str {
        match self {
            Self::Other => "",
            Self::Integrated => " (iGPU)",
            Self::Discrete => " (dGPU)",
            Self::Virtual => " (vGPU)",
            Self::Cpu => " (software)",
        }
    }
}

/// `VulkanPhysicalDevice.vendorName`: a known vendor's id by its name, else the id in hex.
pub fn vendor_name(id: u32) -> String {
    match id {
        0x1002 | 0x1022 => "AMD".to_owned(),
        0x1010 => "IMGTEC".to_owned(),
        0x106B => "APPLE".to_owned(),
        0x10DE | 0x12D2 => "NVIDIA".to_owned(),
        0x13B5 => "ARM".to_owned(),
        0x1414 => "MICROSOFT".to_owned(),
        0x14E4 => "BROADCOM".to_owned(),
        0x168C | 0x17CB | 0x1969 | 0x5143 => "QUALCOMM".to_owned(),
        0x8086 => "INTEL".to_owned(),
        other => format!("{other:#x}"),
    }
}

/// What the entry shows of the machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemSpecs {
    /// The language runtime's name and version: `Java` and `java.version` in the reference.
    pub runtime: (String, String),
    /// `GLX._getCpuInfo`: the logical processors and the processor's name.
    pub cpu: String,
    /// The framebuffer's size in pixels (`Window.getWidth` and `getHeight`).
    pub display: (i32, i32),
    pub vendor: String,
    pub device_name: String,
    pub device_type: DeviceType,
    pub backend: String,
    /// `DeviceInfo.driverInfo`, of which the first line shows.
    pub driver_info: String,
}

pub fn display(specs: &SystemSpecs) -> Vec<Line> {
    let driver = specs
        .driver_info
        .lines()
        .next()
        .unwrap_or(&specs.driver_info);
    vec![Line::Group(
        "minecraft:system",
        vec![
            format!("{}: {}", specs.runtime.0, specs.runtime.1),
            format!("CPU: {}", specs.cpu),
            format!(
                "Display: {}x{} ({})",
                specs.display.0, specs.display.1, specs.vendor
            ),
            format!("{}{}", specs.device_name, specs.device_type.suffix()),
            format!("{} {driver}", specs.backend),
        ],
    )]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_vendor_has_its_name_and_another_its_id_in_hex() {
        assert_eq!(vendor_name(0x1002), "AMD");
        assert_eq!(vendor_name(0x10DE), "NVIDIA");
        assert_eq!(vendor_name(0x8086), "INTEL");
        assert_eq!(vendor_name(0x1af4), "0x1af4");
    }

    #[test]
    fn the_device_type_follows_the_vulkan_value() {
        assert_eq!(DeviceType::from_vulkan(2), DeviceType::Discrete);
        assert_eq!(DeviceType::from_vulkan(4), DeviceType::Cpu);
        assert_eq!(DeviceType::from_vulkan(9), DeviceType::Other);
    }
}
