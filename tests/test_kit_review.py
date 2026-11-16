import json

from scriptcast.tools.stage import export_kit_review


def test_kit_review_export_filters_and_sorts(tmp_path):
    source = tmp_path / 'kit-review.json'
    source.write_text(json.dumps({'version': 1, 'items': {
        'Later': {'priority': 'P2', 'status': 'change', 'notes': 'later note', 'tags': []},
        'First': {'priority': 'P0', 'status': 'cut', 'notes': 'first note', 'tags': ['motion']},
        'Ignored': {'priority': 'P0', 'status': 'keep', 'notes': 'nope', 'tags': []},
    }}))
    output = tmp_path / 'kit-review.md'
    assert export_kit_review(source, output) == output
    text = output.read_text()
    assert text.index('## First') < text.index('## Later')
    assert 'first note' in text and 'later note' in text
    assert 'Ignored' not in text
