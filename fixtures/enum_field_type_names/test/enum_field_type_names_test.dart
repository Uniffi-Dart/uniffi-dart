import 'package:test/test.dart';

import '../enum_field_type_names.dart';

void main() {
  test('a record with Error inside its name keeps it in a variant field', () {
    final out = roundtripOutcome(
      outcome: ProblemOutcome(InternalServerErrorProblem(detail: "boom")),
    );
    expect(out, isA<ProblemOutcome>());
    expect((out as ProblemOutcome).problem.detail, "boom");
  });

  test('the same record inside a sequence and an optional field', () {
    final many = roundtripOutcome(
      outcome: ProblemsOutcome([
        InternalServerErrorProblem(detail: "a"),
        InternalServerErrorProblem(detail: "b"),
      ]),
    );
    expect((many as ProblemsOutcome).problems.map((p) => p.detail).toList(), [
      "a",
      "b",
    ]);

    final some = roundtripOutcome(
      outcome: MaybeProblemOutcome(InternalServerErrorProblem(detail: "c")),
    );
    expect((some as MaybeProblemOutcome).problem?.detail, "c");

    final none = roundtripOutcome(outcome: MaybeProblemOutcome(null));
    expect((none as MaybeProblemOutcome).problem, isNull);
  });

  test('the same record in sequence and optional arguments and returns', () {
    final first = firstProblem(
      problems: [
        InternalServerErrorProblem(detail: "x"),
        InternalServerErrorProblem(detail: "y"),
      ],
    );
    expect(first?.detail, "x");
    expect(firstProblem(problems: []), isNull);
  });

  test('an error type with a trailing Error is still named ...Exception', () {
    final out = roundtripOutcome(
      outcome: LookupOutcome(NotFoundLookupException(7)),
    );
    final error = (out as LookupOutcome).error;
    expect(error, isA<LookupException>());
    expect((error as NotFoundLookupException).id, 7);
  });

  test('an error variant carries the record', () {
    expect(
      () => failWithProblem(detail: "down"),
      throwsA(
        isA<InternalServerApiException>().having(
          (e) => e.problem.detail,
          'problem.detail',
          "down",
        ),
      ),
    );
  });

  test('a fieldless variant is unaffected', () {
    expect(roundtripOutcome(outcome: OkOutcome()), isA<OkOutcome>());
  });

  group('an error type with a trailing Error inside compound types', () {
    int idOf(LookupException? e) => (e as NotFoundLookupException).id;

    test('optional and sequence arguments and returns', () {
      final first = firstLookupError(
        errors: [NotFoundLookupException(1), NotFoundLookupException(2)],
      );
      expect(first, isA<LookupException>());
      expect(idOf(first), 1);
      expect(firstLookupError(errors: []), isNull);

      expect(
        idOf(roundtripMaybeLookupError(error: NotFoundLookupException(3))),
        3,
      );
      expect(roundtripMaybeLookupError(error: null), isNull);

      final all = roundtripLookupErrors(
        errors: [NotFoundLookupException(4), NotFoundLookupException(5)],
      );
      expect(all, isA<List<LookupException>>());
      expect(all.map(idOf).toList(), [4, 5]);
    });

    test('a map argument and return', () {
      final out = roundtripLookupErrorMap(
        errors: {
          "a": NotFoundLookupException(6),
          "b": NotFoundLookupException(7),
        },
      );
      expect(out, isA<Map<String, LookupException>>());
      expect(out.map((k, v) => MapEntry(k, idOf(v))), {"a": 6, "b": 7});
    });

    test('optional, sequence and map record fields', () {
      final out = roundtripLookupReport(
        report: LookupReport(
          first: NotFoundLookupException(8),
          all: [NotFoundLookupException(9)],
          byKey: {"k": NotFoundLookupException(10)},
        ),
      );
      expect(idOf(out.first), 8);
      expect(out.all.map(idOf).toList(), [9]);
      expect(idOf(out.byKey["k"]), 10);

      final empty = roundtripLookupReport(
        report: LookupReport(first: null, all: [], byKey: {}),
      );
      expect(empty.first, isNull);
      expect(empty.all, isEmpty);
      expect(empty.byKey, isEmpty);
    });

    test('optional and sequence variant fields', () {
      final out = roundtripOutcome(
        outcome: LookupsOutcome(
          first: NotFoundLookupException(11),
          all: [NotFoundLookupException(12)],
        ),
      );
      expect(idOf((out as LookupsOutcome).first), 11);
      expect(out.all.map(idOf).toList(), [12]);

      final none = roundtripOutcome(
        outcome: LookupsOutcome(first: null, all: []),
      );
      expect((none as LookupsOutcome).first, isNull);
      expect(none.all, isEmpty);
    });
  });
}
