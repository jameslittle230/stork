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

    for key in all_keys:
        if key in d1 and key in d2:
            out[key] = {
                "baseline": round(d1[key], 4), 
                "contender": round(d2[key], 4),
                "multiplier": round(d2[key] / d1[key], 2)
            }
        elif key not in d1:
            out[key] = {
                "baseline": 0, 
                "contender": round(d2[key], 4),
                "multiplier": 1
            }
        elif key not in d2:
            out[key] = {
                "baseline": round(d1[key], 4), 
                "contender": 0,
                "multiplier": 1
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

