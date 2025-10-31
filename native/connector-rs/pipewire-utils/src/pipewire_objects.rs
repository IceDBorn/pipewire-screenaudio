use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    INPUT,
    OUTPUT,
}

#[derive(Debug)]
pub struct InvalidPortError;
impl FromStr for PortDirection {
    type Err = InvalidPortError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "in" => Ok(PortDirection::INPUT),
            "out" => Ok(PortDirection::OUTPUT),
            _ => Err(InvalidPortError),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChannel<'a> {
    FrontLeft,
    FrontRight,
    Other(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StereoAudioChannel {
    FrontLeft,
    FrontRight,
}

impl<'a> From<&'a str> for AudioChannel<'a> {
    fn from(value: &'a str) -> Self {
        match value {
            "FL" => AudioChannel::FrontLeft,
            "FR" => AudioChannel::FrontRight,
            s => AudioChannel::Other(s),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ports {
    pub fl_port: u32,
    pub fr_port: u32,
}

impl Ports {
    pub fn get_stereo_channel<'a>(&self, channel: &StereoAudioChannel) -> u32 {
        match channel {
            &StereoAudioChannel::FrontLeft => self.fl_port,
            &StereoAudioChannel::FrontRight => self.fr_port,
        }
    }
    pub fn get_channel<'a>(&self, channel: &AudioChannel<'a>) -> Option<&u32> {
        match channel {
            &AudioChannel::FrontLeft => Some(&self.fl_port),
            &AudioChannel::FrontRight => Some(&self.fr_port),
            &AudioChannel::Other(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NodeWithPorts {
    pub id: u32,
    pub ports: Ports,
}

#[derive(Debug, Clone, Copy)]
pub struct MaybePorts {
    pub fl_port: Option<u32>,
    pub fr_port: Option<u32>,
}

impl MaybePorts {
    pub fn both(self) -> Option<Ports> {
        let MaybePorts { fl_port, fr_port } = self;
        let fl_port = fl_port?;
        let fr_port = fr_port?;
        Some(Ports { fl_port, fr_port })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PortInfo<'a> {
    pub(crate) channel: AudioChannel<'a>,
    pub(crate) node_id: u32,
    pub(crate) id: u32,
    pub(crate) direction: PortDirection,
}

#[derive(Error, Debug)]
#[error("port channel is not stereo")]
pub struct ChannelIsNotStereoError;
impl<'a> TryFrom<AudioChannel<'a>> for StereoAudioChannel {
    type Error = ChannelIsNotStereoError;
    fn try_from(value: AudioChannel<'a>) -> Result<Self, Self::Error> {
        match value {
            AudioChannel::FrontLeft => Ok(StereoAudioChannel::FrontLeft),
            AudioChannel::FrontRight => Ok(StereoAudioChannel::FrontRight),
            AudioChannel::Other(_) => Err(ChannelIsNotStereoError),
        }
    }
}

pub struct OwnedPortInfo {
    pub(crate) channel: Option<StereoAudioChannel>,
    pub(crate) node_id: u32,
    pub(crate) id: u32,
    #[allow(unused)]
    pub(crate) direction: PortDirection,
}

impl<'a> From<PortInfo<'a>> for OwnedPortInfo {
    fn from(
        PortInfo {
            channel,
            node_id,
            id,
            direction,
        }: PortInfo<'a>,
    ) -> Self {
        OwnedPortInfo {
            id,
            node_id,
            channel: channel.try_into().ok(),
            direction,
        }
    }
}
