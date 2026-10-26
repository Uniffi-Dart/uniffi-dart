import 'package:test/test.dart';
import '../argument_rollback.dart';

class LoggingPacket implements Packet {
  final bool debug;
  LoggingPacket(this.debug);
  @override
  List<Tracked> get objects => [];
  @override
  int get small {
    final error = DetailFormatException('diagnostic');
    expect(UniffiArgumentScope.current, isNotNull);
    expect(debug ? error.debugString() : error.toString(), contains('diagnostic'));
    return 0;
  }
}
void main() {
  for (final debug in [false, true]) {
    test('${debug ? "Debug" : "Display"} has its own argument ownership scope', () {
      final before = liveBytes();
      for (var i = 0; i < 50; i++) {
        expect(() => takePacket(packet: LoggingPacket(debug), small: 256), throwsArgumentError);
      }
      expect(liveBytes(), before);
    });
  }
}
