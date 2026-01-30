const Database = require('better-sqlite3');
const db = new Database('C:/Users/Administrator/.cc-switch/cc-switch.db', { readonly: true });

// 获取所有表
const tables = db.prepare("SELECT name FROM sqlite_master WHERE type='table'").all();
console.log('Tables:', tables.map(t => t.name));

// 查看每个表的结构和数据
tables.forEach(t => {
  console.log('\n--- Table:', t.name, '---');
  const cols = db.prepare('PRAGMA table_info(' + t.name + ')').all();
  console.log('Columns:', cols.map(c => c.name).join(', '));
  
  // 读取前5行数据
  try {
    const rows = db.prepare('SELECT * FROM ' + t.name + ' LIMIT 5').all();
    console.log('Sample data:', JSON.stringify(rows, null, 2));
  } catch (e) {
    console.log('Error reading:', e.message);
  }
});

db.close();
