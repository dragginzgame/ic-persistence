# Consumer archive admission over already-observed lock and metadata inputs.
def admit_selected_lock($selected):
    def identity: {name, version, source, checksum};
    ($selected.package | map(select(.source != null) | identity)) as $allowed |
    [.package[] | select(.source != null) | identity | select(. as $p | $allowed | index($p) | not)] |
    if length == 0 then true else error("archive dependency selection drift: " + tostring) end;

def admit_extracted_packages($extracted; $fixture):
    . as $graph |
    (all($extracted[]; . as $expected |
        [$graph.packages[] | select(.name == $expected.name)] |
        length == 1 and .[0].manifest_path == $expected.manifest and .[0].source == null) and
    all(.packages[]; .source != null or .manifest_path == $fixture or
        (.manifest_path as $path | any($extracted[]; .manifest == $path)))) |
    if . then true else error("archive graph must use exactly the extracted Persistence packages") end;
