use axum::{
    extract::{Form, State},
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::fs;
use tower_cookies::{Cookie, Cookies, CookieManagerLayer};

// 1. STRUKTUR DATA (Disesuaikan untuk CV BUS)
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Barang {
    nama: String,
    stok: u32,
}

#[derive(Deserialize)]
struct InputBarang {
    nama: String,
    stok: u32,
}

struct AppState {
    inventaris: Mutex<Vec<Barang>>,
}

// 2. LOGIKA UTAMA (Startup & Server)
#[tokio::main]
async fn main() {
    // Memuat data dari gudang.json saat aplikasi dijalankan
    let data_awal = if let Ok(konten) = fs::read_to_string("gudang.json") {
        println!("[SISTEM] Data CV BUS berhasil dimuat dari disk.");
        serde_json::from_str(&konten).unwrap_or_else(|_| initial_data())
    } else {
        println!("[SISTEM] File tidak ditemukan, menggunakan data default.");
        initial_data()
    };

    let shared_state = Arc::new(AppState {
        inventaris: Mutex::new(data_awal),
    });

    let app = Router::new()
        .route("/", get(tampilkan_dashboard))
        .route("/login", get(halaman_login).post(proses_login))
        .route("/tambah", post(proses_input))
        .route("/hapus/:nama", get(proses_hapus))
        .route("/cetak", get(cetak_laporan))
        .layer(CookieManagerLayer::new())
        .with_state(shared_state);

// Mengambil port dari environment variable (standar cloud) atau default ke 8080
let port = std::env::var("PORT")
    .unwrap_or_else(|_| "8080".to_string())
    .parse::<u16>()
    .unwrap();

let addr = SocketAddr::from(([0, 0, 0, 0], port)); 
println!("\n--- SISTEM CV BUS GOES LIVE ---");
println!("Server berjalan pada port: {}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// 3. FUNGSI HANDLER (Web Logic)

async fn tampilkan_dashboard(
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    // PROTEKSI: Cek apakah user sudah login melalui cookies
    if cookies.get("auth").is_none() {
        return Html("<script>window.location.href='/login';</script>".to_string());
    }

    let gudang = state.inventaris.lock().unwrap();
    let mut rows = String::new();
    for item in gudang.iter() {
        rows.push_str(&format!(
            "<tr><td>{}</td><td class='text-center'>{} unit</td></tr>",
            item.nama, item.stok
        ));
    }

    for item in gudang.iter() {
    rows.push_str(&format!(
        "<tr>
            <td>{}</td>
            <td class='text-center'>{} unit</td>
            <td class='text-end'>
                <a href='/hapus/{}' class='btn btn-danger btn-sm' onclick=\"return confirm('Yakin hapus?')\">Hapus</a>
            </td>
        </tr>",
        item.nama, item.stok, item.nama
    ));
}

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html lang="id">
        <head>
            <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css" rel="stylesheet">
            <title>CV BUS - Dashboard</title>
            <style> body {{ background-color: #f8f9fa; }} .card {{ border: none; box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1); }} </style>
        </head>
        <body>
<nav class="navbar navbar-light bg-warning mb-4 shadow-sm">
    <div class="container">
        <span class="navbar-brand mb-0 h1 fw-bold text-dark">
            🚧 CV Berkah Utama Saftindo - Inventory v1.0
        </span>
    </div>
</nav>            <div class="container">
                <div class="row g-4">
                    <div class="col-md-4">
                        <div class="card p-4">
                            <h5>Input Barang</h5>
                            <form action="/tambah" method="post">
                                <input type="text" name="nama" class="form-control mb-2" placeholder="Nama Alat" required>
                                <input type="number" name="stok" class="form-control mb-3" placeholder="Jumlah" required>
                                <button type="submit" class="btn btn-primary w-100">Tambah ke Gudang</button>
                            </form>
                        </div>
                    </div>
                    <div class="col-md-8">
                        <div class="card p-4">
                            <div class="d-flex justify-content-between mb-3">
                                <h5>Daftar Stok Inventaris</h5>
                                <a href="/cetak" class="btn btn-outline-secondary btn-sm">Cetak Laporan (.txt)</a>
                            </div>
                            <table class="table table-hover">
                                <thead class="table-light"><tr><th>Nama Barang</th><th class="text-center">Jumlah</th></tr></thead>
                                <tbody>{}</tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </body>
        </html>"#,
        rows
    ))
}

async fn halaman_login() -> Html<String> {
    Html(r#"
        <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css" rel="stylesheet">
        <body class="bg-light"><div class="container mt-5"><div class="row justify-content-center"><div class="col-md-4 card p-4 shadow-sm text-center">
            <h3>Login Admin BUS</h3>
            <form action="/login" method="post">
                <input type="password" name="password" class="form-control my-3" placeholder="Password: adminBUS2026" required>
                <button type="submit" class="btn btn-dark w-100">Masuk</button>
            </form>
        </div></div></div></body>"#.to_string())
}

async fn proses_login(
    cookies: Cookies,
    Form(input): Form<std::collections::HashMap<String, String>>,
) -> Redirect {
    if input.get("password").map(|p| p == "adminBUS2026").unwrap_or(false) {
        cookies.add(Cookie::new("auth", "true"));
        Redirect::to("/")
    } else {
        Redirect::to("/login")
    }
}

async fn proses_input(
    State(state): State<Arc<AppState>>,
    Form(input): Form<InputBarang>,
) -> Redirect {
    let mut gudang = state.inventaris.lock().unwrap();
    gudang.push(Barang { nama: input.nama, stok: input.stok });
    
    // Simpan permanen ke JSON
    let json = serde_json::to_string(&*gudang).unwrap();
    fs::write("gudang.json", json).expect("Gagal simpan data");
    
    Redirect::to("/")
}

async fn cetak_laporan(State(state): State<Arc<AppState>>) -> Html<String> {
    let gudang = state.inventaris.lock().unwrap();
    let mut isi = String::from("--- LAPORAN STOK CV BUS ---\n\n");
    for (i, b) in gudang.iter().enumerate() {
        isi.push_str(&format!("{}. {} | {} unit\n", i + 1, b.nama, b.stok));
    }
    fs::write("laporan_stok.txt", isi).expect("Gagal cetak");
    Html("<h3>Laporan Berhasil Dicetak ke laporan_stok.txt! <a href='/'>Kembali</a></h3>".to_string())
}

async fn proses_hapus(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(nama): axum::extract::Path<String>,
) -> Redirect {
    let mut gudang = state.inventaris.lock().unwrap();
    
    // Logika penghapusan berdasarkan nama
    gudang.retain(|b| b.nama != nama);
    
    // Update file JSON agar perubahan permanen
    let json = serde_json::to_string(&*gudang).unwrap();
    fs::write("gudang.json", json).expect("Gagal update data");
    
    Redirect::to("/")
}

fn initial_data() -> Vec<Barang> {
    vec![
        Barang { nama: "Helm Proyek Putih".into(), stok: 50 },
        Barang { nama: "Rompi Safety Orange".into(), stok: 120 },
    ]
}