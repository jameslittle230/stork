import os
import sys
import json

with open(sys.argv[1]) as baseline_f, open(sys.argv[2]) as contender_f:
    baseline_d = json.load(baseline_f)
    contender_d = json.load(contender_f)

def generate_stats_dict(d1, d2):
    s_keys_d1 = set(d1.keys())
    s_keys_d2 = set(d2.keys())
    all_keys = list(s_keys_d1.union(s_keys_d2))
    out = dict()

    # TODO: Handle if d1 or d2 doesn't contain key
    for key in all_keys:
        out[key] = {
            "baseline": round(d1[key], 4), 
            "contender": round(d2[key], 4),
            "multiplier": round(d2[key] / d1[key], 2)
        }
    return out

stats = generate_stats_dict(baseline_d, contender_d)

output = "<table><thead><th>Benchmark</th><th>Baseline</th><th>Contender</th><th>Comparison</th></thead><tbody>"

for key in sorted(list(stats.keys())):
    icon = ""
    if stats[key]["multiplier"] > 1.25:
        icon = "⚠️"
    if stats[key]["multiplier"] < 1.0:
        icon = "🎉"
    output += f"""<tr><td><code>{key}</code></td>
    <td>{stats[key]["baseline"]}</td>
    <td>{stats[key]["contender"]}</td>
    <td>{stats[key]["multiplier"]}× {icon}</td></tr>""".replace("    ", "").replace("\n", "")

output += "</tbody></table>"

print(output)

