# 🧮 TermiSQL

**TermiSQL** is an asynchronous Rust-based Terminal User Interface (TUI) for browsing database tables.  
It supports **SQLite**, **MySQL**, and **MariaDB**, providing a fast, lightweight, and responsive experience directly from your terminal.

---

## ⚙️ Installation

### From Crates.io
```bash
cargo install termisql
```

### From Git Repository

```bash
cargo install --git https://github.com/Ellen-desu/termisql
```

## 🚀 Usage

### For SQLite
```bash
termisql sqlite <FILENAME>
```

### For MySQL/MariaDB
```bash
termisql [mysql|mariadb] <DATABASE>
```

## 🧭 Interface Controls


| Key       | Action                                        |
| :-------- | :-------------------------------------------- |
| **Enter** | Switch between *viewer mode* and *focus mode* |
| **← / →** | Move between widgets when in *focus mode*     |
| **↑ / ↓** | Navigate inside the active widget             |
| **Esc**   | Return to *viewer mode* from *focus mode*     |
| **q**     | Quit TermiSQL                                 |

>💡 Tip: When you first open TermiSQL, you're in viewer mode. Press Enter to start interacting with the interface.


## 🧩 Features

- Cross-database support: SQLite, MySQL, MariaDB
- Automatic table rendering with scrollable view
- Keyboard navigation optimized for terminal users
- Built with async Rust (tokio runtime)
- Focus mode for interactive exploration

## 🔧 Planned Features

- PostgreSQL support
- Data export (CSV/JSON)
