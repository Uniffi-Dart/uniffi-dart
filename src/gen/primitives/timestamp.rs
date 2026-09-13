use genco::lang::dart;

use super::paste;
use crate::gen::quote;
use crate::gen::render::{Renderable, TypeHelperRenderer};

impl_code_type_for_primitive!(TimestampCodeType, "DateTime", "Timestamp");

impl Renderable for TimestampCodeType {
    fn render_type_helper(&self, _type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
        quote! {
            // UniFFI timestamps use signed offset seconds and unsigned nanoseconds.
            // Dart exposes UTC DateTime values, truncating submicrosecond precision
            // toward the Unix epoch. Converters borrow incoming RustBuffers.
            class FfiConverterTimestamp {
                static DateTime lift(RustBuffer buf) {
                    return read(buf.asUint8List()).value;
                }

                static RustBuffer lower(DateTime value) {
                    final buf = Uint8List(12);
                    write(value, buf);
                    return toRustBuffer(buf);
                }

                static LiftRetVal<DateTime> read(Uint8List buf) {
                    if (buf.length < 12) {
                        throw FormatException("Timestamp requires 12 bytes");
                    }
                    final bytes = buf.buffer.asByteData(buf.offsetInBytes, 12);
                    final seconds = bytes.getInt64(0);
                    final nanos = bytes.getUint32(8);
                    if (nanos >= 1000000000) {
                        throw FormatException("Invalid timestamp nanoseconds");
                    }
                    // Check seconds before multiplying: Dart int arithmetic wraps.
                    const maxSeconds = 8640000000000;
                    if (seconds < -maxSeconds || seconds > maxSeconds ||
                        (seconds.abs() == maxSeconds && nanos != 0)) {
                        throw RangeError("Timestamp outside DateTime range");
                    }
                    final magnitude = seconds.abs() * 1000000 + nanos ~/ 1000;
                    final micros = seconds < 0 ? -magnitude : magnitude;
                    return LiftRetVal(
                        DateTime.fromMicrosecondsSinceEpoch(micros, isUtc: true), 12);
                }

                static int allocationSize([DateTime? value]) => 12;

                static int write(DateTime value, Uint8List buf) {
                    if (buf.length < 12) {
                        throw RangeError("Timestamp requires 12 bytes");
                    }
                    final micros = value.microsecondsSinceEpoch;
                    // UniFFI stores the sign only in whole seconds. A negative
                    // subsecond offset would otherwise be encoded as a positive one.
                    if (micros < 0 && micros > -1000000) {
                        throw RangeError(
                            "UniFFI cannot represent timestamps less than one second before the Unix epoch");
                    }
                    final bytes = buf.buffer.asByteData(buf.offsetInBytes, 12);
                    final magnitude = micros.abs();
                    final seconds = magnitude ~/ 1000000;
                    bytes.setInt64(0, micros < 0 ? -seconds : seconds);
                    bytes.setUint32(8, (magnitude % 1000000) * 1000);
                    return 12;
                }
            }
        }
    }
}
