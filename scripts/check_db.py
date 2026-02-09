import sqlite3
import sys
import os

dbs = {
    "Trae CN": r"C:\Users\Administrator\AppData\Roaming\Trae CN\User\globalStorage\state.vscdb",
    "Trae": r"C:\Users\Administrator\AppData\Roaming\Trae\User\globalStorage\state.vscdb",
    "Windsurf": r"C:\Users\Administrator\AppData\Roaming\Windsurf\User\globalStorage\state.vscdb",
}

for name, path in dbs.items():
    print(f"\n=== {name} ({path}) ===")
    if not os.path.exists(path):
        print("  FILE NOT FOUND")
        continue
    try:
        c = sqlite3.connect(f"file:{path}?mode=ro", uri=True)
        tables = [r[0] for r in c.execute("SELECT name FROM sqlite_master WHERE type='table'")]
        print(f"  Tables: {tables}")
        
        if "ItemTable" in tables:
            keys = [r[0] for r in c.execute("SELECT key FROM ItemTable LIMIT 30")]
            print(f"  ItemTable keys ({len(keys)}): {keys[:15]}")
            chat_keys = [k for k in keys if "chat" in k.lower() or "composer" in k.lower() or "aichat" in k.lower() or "bubble" in k.lower()]
            if chat_keys:
                print(f"  Chat-related keys: {chat_keys}")
        
        if "cursorDiskKV" in tables:
            keys = [r[0] for r in c.execute("SELECT key FROM cursorDiskKV LIMIT 30")]
            print(f"  cursorDiskKV keys ({len(keys)}): {keys[:15]}")
            chat_keys = [k for k in keys if "composer" in k.lower() or "bubble" in k.lower() or "chat" in k.lower()]
            if chat_keys:
                print(f"  Chat-related keys: {chat_keys}")
        
        # Check for other tables with chat-like data
        for table in tables:
            if table not in ["ItemTable", "cursorDiskKV"]:
                try:
                    count = c.execute(f"SELECT COUNT(*) FROM [{table}]").fetchone()[0]
                    if count > 0:
                        cols = [d[1] for d in c.execute(f"PRAGMA table_info([{table}])")]
                        print(f"  Table [{table}]: {count} rows, cols={cols}")
                except:
                    pass
        c.close()
    except Exception as e:
        print(f"  ERROR: {e}")
