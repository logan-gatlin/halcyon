use super::*;

pub trait IntoLog {
    type OutT;
    fn into_log(
        self,
        logger: &mut FileLogger,
        span: Span,
    ) -> Self::OutT;
}

impl<T, E> IntoLog for Result<T, E>
where
    E: IntoLog<OutT = ()>,
{
    type OutT = Option<T>;

    fn into_log(
        self,
        logger: &mut FileLogger,
        span: Span,
    ) -> Self::OutT {
        match self {
            Ok(t) => Some(t),
            Err(e) => {
                e.into_log(logger, span);
                None
            }
        }
    }
}

impl IntoLog for std::num::ParseIntError {
    type OutT = ();

    fn into_log(
        self,
        logger: &mut FileLogger,
        span: Span,
    ) -> Self::OutT {
        use std::num::IntErrorKind::*;
        let err = logger.error("Unable to parse integer");
        match self.kind() {
            Empty => err.primary("Expected digits here, but found nothing", span),
            InvalidDigit => err.primary("Invalid digits in this integer", span),
            PosOverflow => {
                err.primary("This integer is too large to represent", span)
                    .note("Integers are signed 64 bit values")
            }
            NegOverflow => {
                err.primary("This integer is too small to represent", span)
                    .note("Integers are signed 64 bit values")
            }
            Zero => err.primary("This integer was expected to be non-zero", span),
            _ => err.primary("This integer is invalid", span),
        }
        .done();
    }
}

impl IntoLog for std::num::ParseFloatError {
    type OutT = ();

    fn into_log(
        self,
        logger: &mut FileLogger,
        span: Span,
    ) -> Self::OutT {
        logger
            .error("Unable to parse real number")
            .primary("This real number is invalid", span)
            .done();
    }
}
