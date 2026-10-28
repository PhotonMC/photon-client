use super::*;

/// `ClientboundInitializeBorderPacket`: the whole border state of a level.
pub fn parse_initialize_border_packet(
    input: &[u8],
) -> Result<photon_core::border_init::ClientWorldBorderInitialization, ConnectionError> {
    const NAME: &str = "initialize border";
    let mut input = input;
    let input = &mut input;
    let center_x = read_f64(input, NAME)?;
    let center_z = read_f64(input, NAME)?;
    let old_size = read_f64(input, NAME)?;
    let new_size = read_f64(input, NAME)?;
    let lerp_time =
        photon_core::wire::read_varlong(input).map_err(|_| ConnectionError::Codec(NAME))?;
    let absolute_max_size = read_varint(input, NAME)?;
    let warning_blocks = read_varint(input, NAME)?;
    let warning_time = read_varint(input, NAME)?;
    Ok(photon_core::border_init::ClientWorldBorderInitialization {
        center_x,
        center_z,
        old_size,
        new_size,
        lerp_time,
        absolute_max_size,
        warning_blocks,
        warning_time,
    })
}

/// `ClientboundSetBorderLerpSizePacket`: the size it moves from and to, and over how many ticks.
pub fn parse_set_border_lerp_size_packet(input: &[u8]) -> Result<(f64, f64, i64), ConnectionError> {
    const NAME: &str = "set border lerp size";
    let mut input = input;
    let input = &mut input;
    let old_size = read_f64(input, NAME)?;
    let new_size = read_f64(input, NAME)?;
    let time = photon_core::wire::read_varlong(input).map_err(|_| ConnectionError::Codec(NAME))?;
    Ok((old_size, new_size, time))
}

/// `ClientboundSetBorderCenterPacket`.
pub struct SetBorderCenter {
    pub x: f64,
    pub z: f64,
}

impl Decode for SetBorderCenter {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        const NAME: &str = "set border center";
        let mut input = payload;
        let x = read_f64(&mut input, NAME)?;
        let z = read_f64(&mut input, NAME)?;
        Ok(Self { x, z })
    }
}

/// `ClientboundSetBorderSizePacket`.
pub struct SetBorderSize {
    pub size: f64,
}

impl Decode for SetBorderSize {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        let size = read_f64(&mut input, "set border size")?;
        Ok(Self { size })
    }
}

/// `ClientboundSetBorderWarningDelayPacket`: the warning's time, in seconds.
pub struct SetBorderWarningDelay {
    pub time: i32,
}

impl Decode for SetBorderWarningDelay {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        let time = read_varint(&mut input, "set border warning delay")?;
        Ok(Self { time })
    }
}

/// `ClientboundSetBorderWarningDistancePacket`: the warning's distance, in blocks.
pub struct SetBorderWarningDistance {
    pub blocks: i32,
}

impl Decode for SetBorderWarningDistance {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        let blocks = read_varint(&mut input, "set border warning distance")?;
        Ok(Self { blocks })
    }
}

/// `ClientboundInitializeBorderPacket`.
pub struct InitializeBorder(pub photon_core::border_init::ClientWorldBorderInitialization);

impl Decode for InitializeBorder {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        parse_initialize_border_packet(payload).map(Self)
    }
}

/// `ClientboundSetBorderLerpSizePacket`.
pub struct SetBorderLerpSize {
    pub old_size: f64,
    pub new_size: f64,
    pub time: i64,
}

impl Decode for SetBorderLerpSize {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let (old_size, new_size, time) = parse_set_border_lerp_size_packet(payload)?;
        Ok(Self {
            old_size,
            new_size,
            time,
        })
    }
}
