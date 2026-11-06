import time, laya_mlx as laya
a = laya.load("aac6fef/laya-mlx")
q = {"lane": {"type": "choice", "instructions": "Which worker lane should handle this video-scene task?",
  "criteria": {"svg_asset": "draw an icon or simple vector illustration", "tsx_scene": "write or fix a React/Remotion scene component with animation and layout",
               "script_prose": "rewrite narration or on-screen wording", "audit": "check many files for consistency"}},
  "resume": {"type": "noul", "instructions": "Is this a follow-up fix to work an existing worker session already did?"}}
tasks = ["Fix the overlap between the definition card and the tray in APAGroupDefinitions turn 21; the builder left the tray 40px too high.",
         "Draw a head-and-shoulders person icon in the house palette.",
         "Rewrite the caption for turn 34 so it uses no antithesis.",
         "Check every scene in minutes 3-7 for equation numbers that do not match Paper/The_Original_Power.aux."]
a.predict(tasks[0], q)
for t in tasks:
    s=time.perf_counter(); r=a.predict(t, q); ms=(time.perf_counter()-s)*1000
    an=r["answers"]; print(f"{ms:6.1f}ms", an["lane"].get("choice"), round(an["lane"].get("confidence",0),2), "resume=",round(an["resume"].get("noul",0),2), "|", t[:60])
