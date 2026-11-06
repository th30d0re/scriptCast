"""Label review page. Run: python3 experiments/router_eval/review.py  -> http://localhost:8765

Shows each pre-filled cue card with four dropdowns. Changing a value or pressing
"Looks right" marks the row reviewed and saves labels.jsonl immediately.
"""
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

LABELS = Path(__file__).parent / "labels.jsonl"
CHOICES = {
    "lane": ["tsx_scene", "figcraft_layout", "svg_asset", "on_screen_text", "audit_sweep", "research"],
    "tier": ["mechanical", "standard", "judgment"],
    "session_action": ["resume_builder", "resume_checker", "spawn"],
    "context_source": ["book", "chapter", "persuasion", "none"],
}
HELP = {
    "lane": "What kind of work is it?",
    "tier": "How much model does it need? mechanical = small/cheap, judgment = strongest",
    "session_action": "Continue an existing worker session, or start fresh?",
    "context_source": "Which notebook should feed the brief?",
}

PAGE = """<!doctype html><meta charset=utf-8><title>Router labels</title>
<style>
body{font:14px -apple-system,system-ui;margin:0;background:#0d1520;color:#e8e2d0}
header{position:sticky;top:0;background:#0d1520;padding:12px 24px;border-bottom:1px solid #c9a95b55;display:flex;gap:16px;align-items:center}
.bar{flex:1;height:6px;background:#223;border-radius:3px}.bar i{display:block;height:100%;background:#c9a95b;border-radius:3px}
main{max-width:980px;margin:auto;padding:16px 24px}
.row{border:1px solid #c9a95b33;border-radius:8px;padding:12px 14px;margin:10px 0}
.row.done{opacity:.55}.card{font-size:15px;margin-bottom:6px}.meta{color:#9a9;font-size:12px;margin-bottom:8px}
label{display:inline-block;margin-right:14px;font-size:12px;color:#bba}select{margin-left:4px;background:#162232;color:#eee;border:1px solid #445;border-radius:4px;padding:2px}
button{background:#c9a95b;border:0;border-radius:4px;padding:4px 10px;cursor:pointer}
.filter{color:#bba}</style>
<header><b>Router labels</b><span id=count></span><div class=bar><i id=prog></i></div>
<label class=filter><input type=checkbox id=hide> hide reviewed</label></header>
<main id=list></main>
<script>
const C=__CHOICES__, H=__HELP__; let rows=[];
async function load(){rows=await (await fetch('/labels')).json();draw()}
async function save(r){r.reviewed=true;await fetch('/save',{method:'POST',body:JSON.stringify(r)});draw()}
function draw(){
  const hide=document.getElementById('hide').checked, n=rows.filter(r=>r.reviewed).length;
  document.getElementById('count').textContent=n+' / '+rows.length+' reviewed';
  document.getElementById('prog').style.width=(100*n/rows.length)+'%';
  const L=document.getElementById('list');L.innerHTML='';
  rows.forEach(r=>{ if(hide&&r.reviewed)return;
    const d=document.createElement('div');d.className='row'+(r.reviewed?' done':'');
    d.innerHTML=`<div class=card></div><div class=meta></div>`;
    d.querySelector('.card').textContent=r.card;
    d.querySelector('.meta').textContent=[r.source,r.ran_with&&('ran with '+r.ran_with),r.outcome&&('outcome '+r.outcome),r.note].filter(Boolean).join(' · ');
    for(const k in C){const l=document.createElement('label');l.title=H[k];l.textContent=k;
      const s=document.createElement('select');C[k].forEach(o=>{const e=new Option(o,o);if(r[k]===o)e.selected=true;s.add(e)});
      s.onchange=()=>{r[k]=s.value;save(r)};l.appendChild(s);d.appendChild(l)}
    const b=document.createElement('button');b.textContent=r.reviewed?'Reviewed ✓':'Looks right';b.onclick=()=>save(r);d.appendChild(b);
    L.appendChild(d)})}
document.getElementById('hide').onchange=draw;load();
</script>"""


def read():
    return [json.loads(l) for l in LABELS.read_text().splitlines() if l.strip()]


class H(BaseHTTPRequestHandler):
    def _send(self, body: bytes, ctype: str):
        self.send_response(200)
        self.send_header("Content-Type", ctype)
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        if self.path == "/labels":
            return self._send(json.dumps(read()).encode(), "application/json")
        page = PAGE.replace("__CHOICES__", json.dumps(CHOICES)).replace("__HELP__", json.dumps(HELP))
        self._send(page.encode(), "text/html; charset=utf-8")

    def do_POST(self):
        new = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        rows = [new if r["id"] == new["id"] else r for r in read()]
        LABELS.write_text("".join(json.dumps(r) + "\n" for r in rows))
        self._send(b"ok", "text/plain")

    def log_message(self, *a):
        pass


if __name__ == "__main__":
    print("http://localhost:8765")
    ThreadingHTTPServer(("127.0.0.1", 8765), H).serve_forever()
