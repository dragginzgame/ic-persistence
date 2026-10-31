# Explicit policy, never inferred from directory/package names. Shell admits paths
# and successful locked Cargo output; this projection admits every workspace member.
def name: type == "string" and test("^[A-Za-z0-9][A-Za-z0-9_-]*$");
def relative: type == "string" and length > 0 and
  (test("[\\r\\n\\t]") or startswith("/") or
   (split("/") | any(. == "" or . == "." or . == "..")) | not);
def stable: type == "string" and test("^(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)$");
def admit_groups($config; $root; $manifests; $lock; $selected):
  . as $metadata |
  [.packages[] | select(.id as $id | $metadata.workspace_members | index($id))] as $owned |
  if ($config | keys - ["shared_metadata_files"]) != ["groups", "schema"] or $config.schema != 1 or
     ($config.groups | type) != "array" or ($config.groups | length) == 0 or
     ($owned | length) == 0 or
     ($owned | map(.id) | sort) != ($metadata.workspace_members | sort) or
     ($owned | map(.name) | unique | length) != ($owned | length) or
     any($owned[]; .source != null or (.name | name | not) or (.version | stable | not))
  then error("invalid release-group workspace/configuration") else . end |
  $config.groups as $groups |
  if any($groups[]; (keys - ["metadata_files"]) != ["changelog","dependencies","members","name","primary","version"] or
     (.name | name | not) or (.primary | name | not) or (.changelog | relative | not) or
     (.members | type) != "array" or (.members | length) == 0 or (.members | all(.[]; name) | not))
  then error("invalid release-group declaration") else . end |
  if any($groups[]; . as $g | (.members | index($g.primary)) == null or
       (.version != "workspace" and .version != "package") or
       (.dependencies | type) != "array" or (.dependencies | all(.[]; name) | not) or
       (.dependencies | unique | length) != (.dependencies | length) or
       .changelog == "Cargo.toml" or .changelog == "Cargo.lock" or (.changelog | endswith("/Cargo.toml"))) or
     ($groups | map(.name) | unique | length) != ($groups | length) or
     ($groups | map(.changelog) | unique | length) != ($groups | length) or
     ($groups | map(select(.version == "workspace")) | length) > 1 or
     ([$groups[].members[]] | sort) != ($owned | map(.name) | sort)
  then error("missing/overlapping release-group members or identities") else . end |
  if any($groups[]; has("metadata_files") and
      ((.metadata_files | type) != "array" or (.metadata_files | all(.[]; relative) | not)))
  then error("invalid release-group metadata files") else . end |
  if ($config | has("shared_metadata_files")) and
      (($config.shared_metadata_files | type) != "array" or
       ($config.shared_metadata_files | all(.[]; relative) | not))
  then error("invalid shared release metadata files") else . end |
  ($config.shared_metadata_files // []) as $shared |
  ([$groups[] | (.metadata_files // [])[]] + $shared) as $extra |
  (["Cargo.toml", "Cargo.lock", "ci/cargo-release-groups.json"] +
   [$groups[].changelog] +
   [$owned[].manifest_path | ltrimstr($metadata.workspace_root + "/")]) as $reserved |
  if ($extra | unique | length) != ($extra | length) or
     any($extra[]; . as $path | ($reserved | index($path)) != null or
       $path == ".git" or ($path | startswith(".git/")))
  then error("overlapping or reserved release-group metadata files") else . end |
  ($groups | map(. as $g |
    ($owned | map(select(.name as $n | $g.members | index($n)))) as $members |
    ($members | map(select(.name == $g.primary))[0].version) as $version |
    if any($members[]; . as $p |
       .version != $version or
       (if $g.version == "workspace" then
          $root.workspace.package.version != $version or $manifests[$p.name].package.version != {workspace:true}
        else $manifests[$p.name].package.version != $version end))
    then error("mismatched release-group versions/inheritance")
    else . + {previous:$version, shared_metadata_files:$shared} end)) as $admitted |
  if ([$lock.package[] | select(.source == null) | {name,version}] | sort_by(.name)) !=
     ($owned | map({name,version}) | sort_by(.name))
  then error("stale or foreign local lock records") else . end |
  ($admitted | map(select(.name == $selected))) as $matches |
  if ($matches | length) != 1 then error("unknown release group") else $matches[0] end;

def group_plan($group; $root; $owned; $directory; $next):
  [$group.members[] as $n | $owned[] | select(.name == $n) |
    {name, path:(.manifest_path | ltrimstr($directory + "/"))}] as $members |
  [($root.workspace.dependencies // {} | to_entries[]) |
    select((.value | type) == "object" and .value.path != null and .value.version != null) |
    . as $d | ($d.value.package // $d.key) as $package |
    select($group.members | index($package)) |
    if ($group.dependencies | index($d.key)) == null then error("local requirement change not admitted by group")
    elif ($d.value.version | test("^[~^=]?[ \\t]*(0|[1-9][0-9]*)(\\.(0|[1-9][0-9]*)){0,2}$") | not)
    then error("unsupported selected local requirement")
    else {alias:$d.key, package:$package, path:$d.value.path, previous:$d.value.version,
      next:($d.value.version | capture("^(?<prefix>[~^=]?[ \\t]*)(?<version>.*)$") |
        . as $r | .prefix + ($next | split(".")[:($r.version | split(".") | length)] | join(".")))} end] as $dependencies |
  if ($dependencies | map(.alias) | sort) != ($group.dependencies | sort)
  then error("unknown or nonlocal group dependency selection") else . end |
  (($group.shared_metadata_files // []) + ($group.metadata_files // [])) as $metadata_files |
  {group:$group.name, primary:$group.primary, previous:$group.previous, changelog:$group.changelog,
   members:$group.members, dependencies:$dependencies, metadata_files:$metadata_files,
   versions:($owned | map({key:.name,value:.version}) | from_entries),
   manifests:([{path:"Cargo.toml", version:(if $group.version == "workspace" then "workspace" elif any($members[]; .path == "Cargo.toml") then "package" else "unchanged" end),
     dependencies:($dependencies | map(select(.previous != .next) | {alias,previous,next}))}] +
     (if $group.version == "package" then $members | map(select(.path != "Cargo.toml") | {path,version:"package",dependencies:[]}) else [] end)),
   files:(["Cargo.toml","Cargo.lock",$group.changelog] +
     (if $group.version == "package" then $members | map(.path | select(. != "Cargo.toml")) else [] end) +
      $metadata_files)};
