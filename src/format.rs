//! Defines format behaviour

use std::error::Error;

#[cfg(all(feature = "browser-json", target_arch = "wasm32"))]
pub mod browser_json;
#[cfg(feature = "json")]
pub mod json;
#[cfg(all(feature = "browser-json", not(target_arch = "wasm32")))]
compile_error!("browser-json is only available on wasm32 arch");
#[cfg(feature = "cbor")]
pub mod cbor;

/// This specifies types which represent formats, but does not prescribe supporting any particular type
pub trait IsFormat: Clone + Send + Sync + 'static {
    /// The HTTP content-type related to this format
    fn content_type(&self) -> &'static str;

    /// The string that is used in websocket subprotocol
    fn subprotocol(&self) -> &'static str;
}

/// A format which is able to deserialise `Read` and serialise `Write`
pub trait Format<Read, Write>: ReadFormat<Read> + WriteFormat<Write> + IsFormat {
}

/// A format which is able to deserialise `Read`
pub trait ReadFormat<Read>: Send + Sync + IsFormat {
    /// Read a value from the given [Read]
    fn read(&self, reader: &[u8]) -> Result<Read, Box<dyn Error + Send>>;
}

/// A format which is able to serialise `Write`
pub trait WriteFormat<Write>: Send + Sync + IsFormat {
    /// write the given value to the given [Write]
    fn write(&self, value: Write) -> Result<Vec<u8>, Box<dyn Error + Send>>;
}

/// A format which is either A or B
#[allow(missing_docs, reason = "Variants are self documenting")]
#[derive(Debug, Clone, Copy)]
pub enum EitherFormat<A, B> {
    Left(A),
    Right(B),
}

impl<A, B, Read> ReadFormat<Read> for EitherFormat<A, B>
where
    A: ReadFormat<Read>,
    B: ReadFormat<Read>,
{
    fn read(&self, reader: &[u8]) -> Result<Read, Box<dyn Error + Send>> {
        match self {
            Self::Left(format) => format.read(reader),
            Self::Right(format) => format.read(reader),
        }
    }
}

impl<A, B, Write> WriteFormat<Write> for EitherFormat<A, B>
where
    A: WriteFormat<Write>,
    B: WriteFormat<Write>,
{
    fn write(&self, value: Write) -> Result<Vec<u8>, Box<dyn Error + Send>> {
        match self {
            Self::Left(format) => format.write(value),
            Self::Right(format) => format.write(value),
        }
    }
}

impl<A, B> IsFormat for EitherFormat<A, B>
where
    A: IsFormat,
    B: IsFormat,
{
    fn content_type(&self) -> &'static str {
        match self {
            Self::Left(format) => format.content_type(),
            Self::Right(format) => format.content_type(),
        }
    }

    fn subprotocol(&self) -> &'static str {
        match self {
            Self::Left(format) => format.subprotocol(),
            Self::Right(format) => format.subprotocol(),
        }
    }
}

impl<A, B, Read, Write> Format<Read, Write> for EitherFormat<A, B>
where A: Format<Read, Write>,
    B: Format<Read, Write>,
{

}

/// A set of multiple possible formats, design for servers to support multiple formats,
/// easiest way to create a `FormatSet` is a tuple of formats
pub trait FormatSet: Clone {
    /// The format type outputted by this set
    type Format;

    /// Get a format that matches the type string
    fn for_subprotocol(&self, subprotocol: &impl PartialEq<str>) -> Option<Self::Format>;

    /// Get a list of all format types strings
    fn subprotocols(&self) -> Vec<&'static str>;
}

impl<A: FormatSet> FormatSet for (A,) {
    type Format = A::Format;
    fn for_subprotocol(&self, subprotocol: &impl PartialEq<str>) -> Option<Self::Format> {
        let (a,) = self;
        a.for_subprotocol(subprotocol)
    }

    fn subprotocols(&self) -> Vec<&'static str> {
        let (a,) = self;
        a.subprotocols()
    }
}

impl<A: FormatSet, B: FormatSet> FormatSet for (A, B) {
    type Format = EitherFormat<A::Format, B::Format>;
    fn for_subprotocol(&self, subprotocol: &impl PartialEq<str>) -> Option<Self::Format> {
        let (a, b) = self;
        a.for_subprotocol(subprotocol).map(EitherFormat::Left)
            .or_else(|| b.for_subprotocol(subprotocol).map(EitherFormat::Right))
    }
    fn subprotocols(&self) -> Vec<&'static str> {
        let (a, b) = self;
        let mut result = a.subprotocols();
        result.extend(b.subprotocols());
        result
    }
}

impl<A: FormatSet, B: FormatSet, C: FormatSet> FormatSet for (A, B, C) {
    type Format = <((A, B), C) as FormatSet>::Format;
    fn for_subprotocol(&self, subprotocol: &impl PartialEq<str>) -> Option<Self::Format> {
        let (a, b, c) = self.clone();
        FormatSet::for_subprotocol(&((a, b), c), subprotocol)
    }

    fn subprotocols(&self) -> Vec<&'static str> {
        let (a, b, c) = self.clone();
        FormatSet::subprotocols(&((a, b), c))
    }
}

impl<F: IsFormat> FormatSet for F {
    type Format = F;

    fn for_subprotocol(&self, subprotocol: &impl PartialEq<str>) -> Option<Self::Format> {
        if subprotocol == self.subprotocol() {
            Some(self.clone())
        } else {
            None
        }
    }

    fn subprotocols(&self) -> Vec<&'static str> {
        vec![self.subprotocol()]
    }
}
