use crate::services::serial::crc;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryPayload {
    x_acceleration: f32,
    y_acceleration: f32,
    z_acceleration: f32,
    x_angular_velocity: f32,
    y_angular_velocity: f32,
    z_angular_velocity: f32,
    longitude: f32,
    latitude: f32,
    altitude: f32,
    ground_speed: f32,
    vertical_velocity: f32,
    air_pressure: f32,
    temperature: f32,
}

fn parse_payload(buffer: &[u8]) -> Result<TelemetryPayload, String> {
    if buffer.len() < 52 {
        return Err("buffer too short".to_string());
    }

    //   order is platform-bind, for MCU like ESP32, ...etc
    let x_acceleration = f32::from_be_bytes(buffer[0..4].try_into().unwrap());
    let y_acceleration = f32::from_be_bytes(buffer[4..8].try_into().unwrap());
    let z_acceleration = f32::from_be_bytes(buffer[8..12].try_into().unwrap());
    let x_angular_velocity = f32::from_be_bytes(buffer[12..16].try_into().unwrap());
    let y_angular_velocity = f32::from_be_bytes(buffer[16..20].try_into().unwrap());
    let z_angular_velocity = f32::from_be_bytes(buffer[20..24].try_into().unwrap());
    let longitude = f32::from_be_bytes(buffer[24..28].try_into().unwrap());
    let latitude = f32::from_be_bytes(buffer[28..32].try_into().unwrap());
    let altitude = f32::from_be_bytes(buffer[32..36].try_into().unwrap());
    let ground_speed = f32::from_be_bytes(buffer[36..40].try_into().unwrap());
    let vertical_velocity = f32::from_be_bytes(buffer[40..44].try_into().unwrap());
    let air_pressure = f32::from_be_bytes(buffer[44..48].try_into().unwrap());
    let temperature = f32::from_be_bytes(buffer[48..52].try_into().unwrap());

    Ok(TelemetryPayload {
        x_acceleration,
        y_acceleration,
        z_acceleration,
        x_angular_velocity,
        y_angular_velocity,
        z_angular_velocity,
        longitude,
        latitude,
        altitude,
        ground_speed,
        vertical_velocity,
        air_pressure,
        temperature,
    })
}

pub enum ParseState {
    Header,
    Payload,
    Crc(u8),
}

pub enum ParseResult {
    Incomplete,
    Ok(TelemetryPayload),
    CrcError(String),
}

pub struct Parser {
    pub state: ParseState,
    pub buffer: Vec<u8>,
}

impl Parser {
    pub fn new() -> Self {
        let parser = Parser {
            state: ParseState::Header,
            buffer: vec![],
        };

        parser
    }

    pub fn feed(&mut self, byte: u8) -> ParseResult {
        match self.state {
            ParseState::Header => {
                if byte == 0xAA {
                    self.buffer.clear();
                    self.state = ParseState::Payload;
                }
                ParseResult::Incomplete
            }
            ParseState::Payload => {
                self.buffer.push(byte);
                if self.buffer.len() == 52 {
                    self.state = ParseState::Crc(0);
                }
                ParseResult::Incomplete
            }
            ParseState::Crc(0) => {
                self.state = ParseState::Crc(byte);
                ParseResult::Incomplete
            }
            ParseState::Crc(first) => {
                let crc = (first as u16) << 8 | (byte as u16);
                self.state = ParseState::Header;
                if self.verify(crc) {
                    match parse_payload(&self.buffer) {
                        Ok(p) => ParseResult::Ok(p),
                        Err(_) => ParseResult::CrcError("failed while parsing payload".to_string()),
                    }
                } else {
                    ParseResult::CrcError("failed while verifying CRC".to_string())
                }
            }
        }
    }

    pub fn verify(&self, crc: u16) -> bool {
        crc::crc16_ccitt(&self.buffer) == crc
    }
}
