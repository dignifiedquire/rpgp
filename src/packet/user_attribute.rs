use std::{fmt, io};

use byteorder::{LittleEndian, WriteBytesExt};
use nom::{be_u8, le_u16, rest};

use errors::Result;
use ser::Serialize;
use types::Version;
use util::{packet_length, write_packet_len};

/// User Attribute Packet
/// https://tools.ietf.org/html/rfc4880.html#section-5.12
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UserAttribute {
    Image {
        packet_version: Version,
        header: Vec<u8>,
        data: Vec<u8>,
    },
    Unknown {
        packet_version: Version,
        typ: u8,
        data: Vec<u8>,
    },
}

impl UserAttribute {
    /// Parses a `UserAttribute` packet from the given slice.
    pub fn from_slice(packet_version: Version, input: &[u8]) -> Result<Self> {
        let (_, pk) = parse(input, packet_version)?;

        Ok(pk)
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            UserAttribute::Image { .. } => 1,
            UserAttribute::Unknown { typ, .. } => typ,
        }
    }

    pub fn packet_version(&self) -> Version {
        match self {
            UserAttribute::Image { packet_version, .. } => *packet_version,
            UserAttribute::Unknown { packet_version, .. } => *packet_version,
        }
    }

    pub fn packet_len(&self) -> usize {
        match self {
            UserAttribute::Image { ref data, .. } => {
                // typ + image header + data length
                1 + 16 + data.len()
            }
            UserAttribute::Unknown { ref data, .. } => {
                // typ + data length
                1 + data.len()
            }
        }
    }
}

impl fmt::Display for UserAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserAttribute::Image { data, .. } => {
                write!(f, "User Attribute: Image (len: {})", data.len())
            }
            UserAttribute::Unknown { typ, data, .. } => {
                write!(f, "User Attribute: typ: {} (len: {})", typ, data.len())
            }
        }
    }
}

#[rustfmt::skip]
named_args!(image(packet_version: Version) <UserAttribute>, do_parse!(
    // little endian, for historical reasons..
       header_len: le_u16
    >>     header: take!(header_len - 2)
    // the actual image is the rest
    >>         img: rest
    >> (UserAttribute::Image {
        packet_version,
        header: header.to_vec(),
        data: img.to_vec()
    })
));

#[rustfmt::skip]
named_args!(parse(packet_version: Version) <UserAttribute>, do_parse!(
        len: packet_length
    >>  typ: be_u8
    >> attr: flat_map!(
        take!(len-1),
        switch!(value!(typ),
                1 => call!(image, packet_version) |
                _ => map!(rest, |data| UserAttribute::Unknown {
                    packet_version,
                    typ,
                    data: data.to_vec()
                })
        ))
    >> (attr)
));

impl Serialize for UserAttribute {
    fn to_writer<W: io::Write>(&self, writer: &mut W) -> Result<()> {
        write_packet_len(self.packet_len(), writer)?;

        match self {
            UserAttribute::Image {
                ref data,
                ref header,
                ..
            } => {
                // typ: image
                writer.write_all(&[0x01])?;
                writer.write_u16::<LittleEndian>((header.len() + 2) as u16)?;
                writer.write_all(header)?;

                // actual data
                writer.write_all(data)?;
            }
            UserAttribute::Unknown { ref data, typ, .. } => {
                writer.write_all(&[*typ])?;
                writer.write_all(data)?;
            }
        }
        Ok(())
    }
}
