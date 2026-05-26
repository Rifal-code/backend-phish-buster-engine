# Rakshaka API Documentation

> **Backend API** for Rakshaka — a platform for reporting online scams, phishing, and illegal gambling (*judol*) with integrated link safety checking.

---

## Table of Contents

- [Project Setup](#project-setup)
- [Running the Project](#running-the-project)
- [Environment Variables](#environment-variables)
- [Authentication Flow](#authentication-flow)
- [Response Format](#response-format)
- [Auth API](#auth-api)
- [Report API](#report-api)
- [Public Report API](#public-report-api)
  - [GET /reports/public](#get-reportspublic)
  - [GET /reports/public/:id](#get-reportspublicid)
- [Link Checker API](#link-checker-api)
- [Error Responses](#error-responses)

---

## Project Setup

### Prerequisites

- Rust (edition 2021, stable toolchain)
- MySQL or MariaDB database
- `cargo` package manager

### Installation

```bash
# Clone the repository
git clone <repo-url>
cd rakshaka

# Copy and configure environment
cp .env.example .env
# Edit .env with your values

# Run the server
cargo run
```

### Database Setup

The application uses **SQLx** with auto-migration. Ensure your database is running and `DATABASE_URL` is correct. Migrations run automatically on server start.

```sql
-- Database will be created by migrations automatically
-- Tables: users, reports, report_images
```

---

## Running the Project

```bash
# Development mode (with auto-reload via cargo-watch)
cargo watch -x run

# Production build
cargo build --release
./target/release/rakshaka

# Check for compile errors
cargo check

# Run with custom log level
RUST_LOG=debug cargo run
```

The server starts on **`http://0.0.0.0:3000`** by default.

---

## Environment Variables

| Variable | Required | Description |
|---|---|---|
| `DATABASE_URL` | ✅ | MySQL connection string: `mysql://user:pass@host/dbname` |
| `JWT_SECRET` | ✅ | Secret key used to sign/verify JWT tokens |
| `LINK_CHECKER_API_KEY` | ⚠️ | API key for the external link checker service |
| `LINK_CHECKER_API_URL` | ⚠️ | Full URL of the external link checker endpoint |
| `RUST_LOG` | ❌ | Log level: `error`, `warn`, `info`, `debug`, `trace` (default: `info`) |

### Example `.env`

```env
DATABASE_URL=mysql://root:password@localhost/rakshaka
JWT_SECRET=your_super_secret_jwt_key_here
LINK_CHECKER_API_KEY=your_api_key_here
LINK_CHECKER_API_URL=https://api.yourlinkchecker.com/check
RUST_LOG=info
```

> **⚠️ Security:** Never commit `.env` to version control. Add it to `.gitignore`.

---

## Authentication Flow

```
1. POST /auth/register  →  Create account (role: user)
2. POST /auth/login     →  Get JWT token
3. Use token as:        →  Authorization: Bearer <token>
4. GET /auth/me         →  Verify token & get profile
```

JWT tokens contain:
- `sub` — user ID (UUID)
- `role` — `admin` or `user`
- `exp` — expiration timestamp (24 hours)

---

## Response Format

All responses follow a consistent JSON structure.

### Success

```json
{
  "success": true,
  "message": "Human-readable success message",
  "data": { ... }
}
```

### Error

```json
{
  "success": false,
  "message": "Human-readable error message"
}
```

---

## Auth API

### POST /auth/register

Register a new user account.

**Authorization:** None (public)

**Request Body:**

```json
{
  "username": "johndoe",
  "email": "john@example.com",
  "password": "securepassword123"
}
```

**Validation Rules:**
- `username`: 3–50 characters, must be unique
- `email`: valid email format, must be unique
- `password`: minimum 8 characters

**Response `201 Created`:**

```json
{
  "success": true,
  "message": "User registered successfully",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "johndoe",
    "email": "john@example.com",
    "role": "user",
    "created_at": "2026-05-23 12:00:00"
  }
}
```

**cURL Example:**

```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "johndoe",
    "email": "john@example.com",
    "password": "securepassword123"
  }'
```

---

### POST /auth/login

Authenticate and receive a JWT token.

**Authorization:** None (public)

**Request Body:**

```json
{
  "email": "john@example.com",
  "password": "securepassword123"
}
```

**Response `200 OK`:**

```json
{
  "success": true,
  "message": "Login successful",
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "johndoe",
      "email": "john@example.com",
      "role": "user",
      "created_at": "2026-05-23 12:00:00"
    }
  }
}
```

**cURL Example:**

```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@example.com",
    "password": "securepassword123"
  }'
```

---

### GET /auth/me

Get the authenticated user's profile.

**Authorization:** `Bearer <token>` ✅ Required

**Response `200 OK`:**

```json
{
  "success": true,
  "message": "User retrieved",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "johndoe",
    "email": "john@example.com",
    "role": "user",
    "created_at": "2026-05-23 12:00:00"
  }
}
```

**cURL Example:**

```bash
curl http://localhost:3000/auth/me \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

---

## Report API

All report endpoints require authentication.

### GET /reports

List reports. Users see only their own reports. Admins see all reports.

**Authorization:** `Bearer <token>` ✅ Required

**Query Parameters:**

| Parameter | Type | Default | Description |
|---|---|---|---|
| `page` | integer | `1` | Page number (starts at 1) |
| `per_page` | integer | `10` | Items per page (max 100) |

**Response `200 OK`:**

```json
{
  "success": true,
  "message": "Reports retrieved",
  "data": {
    "data": [
      {
        "id": "report-uuid-here",
        "user_id": "user-uuid-here",
        "title": "Suspicious Investment Website",
        "description": "This website promises unrealistic returns...",
        "category": "scam",
        "created_at": "2026-05-23 12:00:00",
        "updated_at": "2026-05-23 12:00:00",
        "images": [
          {
            "id": "image-uuid-here",
            "image_url": "https://storage.example.com/image1.jpg"
          }
        ]
      }
    ],
    "page": 1,
    "per_page": 10,
    "total": 42,
    "total_pages": 5
  }
}
```

**cURL Example:**

```bash
curl "http://localhost:3000/reports?page=1&per_page=10" \
  -H "Authorization: Bearer <token>"
```

---

### GET /reports/:id

Get a single report by ID.

**Authorization:** `Bearer <token>` ✅ Required  
**Ownership:** Users can only view their own reports. Admins can view all.

**Response `200 OK`:**

```json
{
  "success": true,
  "message": "Report retrieved",
  "data": {
    "id": "report-uuid-here",
    "user_id": "user-uuid-here",
    "title": "Phishing Email Campaign",
    "description": "Received an email pretending to be from BCA bank...",
    "category": "phishing",
    "created_at": "2026-05-23 12:00:00",
    "updated_at": "2026-05-23 12:00:00",
    "images": []
  }
}
```

**cURL Example:**

```bash
curl http://localhost:3000/reports/report-uuid-here \
  -H "Authorization: Bearer <token>"
```

---

### POST /reports

Create a new report.

**Authorization:** `Bearer <token>` ✅ Required

**Request Body:**

```json
{
  "title": "Illegal Gambling Site",
  "description": "This site operates slot machines targeting Indonesian users without license...",
  "category": "judol",
  "images": [
    "https://storage.example.com/screenshot1.jpg",
    "https://storage.example.com/screenshot2.jpg"
  ]
}
```

**Validation Rules:**
- `title`: 3–200 characters, required
- `description`: minimum 10 characters, required
- `category`: must be one of `scam`, `phishing`, `judol`
- `images`: optional, maximum **3** image URLs

**Response `201 Created`:**

```json
{
  "success": true,
  "message": "Report created successfully",
  "data": {
    "id": "new-report-uuid",
    "user_id": "user-uuid",
    "title": "Illegal Gambling Site",
    "description": "This site operates slot machines...",
    "category": "judol",
    "created_at": "2026-05-23 12:00:00",
    "updated_at": "2026-05-23 12:00:00",
    "images": [
      { "id": "img-uuid-1", "image_url": "https://storage.example.com/screenshot1.jpg" },
      { "id": "img-uuid-2", "image_url": "https://storage.example.com/screenshot2.jpg" }
    ]
  }
}
```

**cURL Example:**

```bash
curl -X POST http://localhost:3000/reports \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Illegal Gambling Site",
    "description": "This site operates slot machines targeting Indonesian users without license...",
    "category": "judol",
    "images": ["https://example.com/screenshot.jpg"]
  }'
```

---

### PUT /reports/:id

Update an existing report.

**Authorization:** `Bearer <token>` ✅ Required  
**Ownership:** Users can only update their own reports. Admins can update any report.

**Request Body:** (all fields optional)

```json
{
  "title": "Updated Title",
  "description": "Updated description with more details...",
  "category": "scam",
  "images": ["https://storage.example.com/new-screenshot.jpg"]
}
```

**Response `200 OK`:**

```json
{
  "success": true,
  "message": "Report updated successfully",
  "data": { ... }
}
```

**cURL Example:**

```bash
curl -X PUT http://localhost:3000/reports/report-uuid-here \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated Report Title",
    "category": "phishing"
  }'
```

---

### DELETE /reports/:id

Delete a report.

**Authorization:** `Bearer <token>` ✅ Required  
**Ownership:** Users can only delete their own reports. Admins can delete any report.

**Response `200 OK`:**

```json
{
  "success": true,
  "message": "Report deleted successfully",
  "data": null
}
```

**cURL Example:**

```bash
curl -X DELETE http://localhost:3000/reports/report-uuid-here \
  -H "Authorization: Bearer <token>"
```

---

## Public Report API

Endpoint ini **tidak membutuhkan autentikasi**. Dirancang untuk homepage, landing page, dan public feed frontend.

### GET /reports/public

Ambil semua laporan terbaru secara publik. Tidak ada data sensitif yang di-expose.

**Authorization:** ❌ Tidak diperlukan (public)

**Query Parameters:**

| Parameter | Type | Default | Max | Description |
|---|---|---|---|---|
| `page` | integer | `1` | — | Halaman yang diminta (mulai dari 1) |
| `per_page` | integer | `10` | `50` | Jumlah item per halaman |

**Response `200 OK`:**

```json
{
  "success": true,
  "message": "Public reports retrieved",
  "data": {
    "data": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "title": "Phishing link menyamar sebagai BCA",
        "description": "Link palsu tersebar menargetkan nasabah BCA melalui WhatsApp...",
        "category": "phishing",
        "created_at": "2026-05-25 20:00:00",
        "images": [
          {
            "id": "img-uuid-1",
            "image_url": "https://storage.example.com/phishing-screenshot.jpg"
          }
        ]
      },
      {
        "id": "661f9511-f30c-52e5-b827-557766551111",
        "title": "Situs judi online ilegal muncul di iklan",
        "description": "Situs judol ini beriklan di platform media sosial tanpa izin...",
        "category": "judol",
        "created_at": "2026-05-25 18:30:00",
        "images": []
      }
    ],
    "page": 1,
    "per_page": 10,
    "total": 42,
    "total_pages": 5
  }
}
```

**Field yang ditampilkan:**

| Field | Type | Keterangan |
|---|---|---|
| `id` | string (UUID) | ID laporan |
| `title` | string | Judul laporan |
| `description` | string | Deskripsi lengkap |
| `category` | string | `scam` \| `phishing` \| `judol` |
| `created_at` | string (datetime) | Waktu dibuat |
| `images` | array | Maks. 3 gambar per laporan |

**Field yang TIDAK ditampilkan** (sengaja disembunyikan):
- `user_id` — tidak bisa trace laporan ke pemilik
- `updated_at` — tidak relevan untuk tampilan publik
- email, password, atau data user apapun

**cURL Example:**

```bash
# Halaman pertama (default)
curl http://localhost:3000/reports/public

# Halaman 2, 5 item per halaman
curl "http://localhost:3000/reports/public?page=2&per_page=5"
```

**JavaScript (fetch) Example:**

```javascript
const res = await fetch('http://localhost:3000/reports/public?page=1&per_page=10');
const json = await res.json();
console.log(json.data.data); // array of reports
```

> **Catatan:** Data diurutkan dari terbaru ke terlama (`ORDER BY created_at DESC`).

---

### GET /reports/public/:id

Ambil detail satu laporan berdasarkan ID. Tidak ada autentikasi yang diperlukan.

**Authorization:** ❌ Tidak diperlukan (public)

**Path Parameter:**

| Parameter | Type | Description |
|---|---|---|
| `id` | string (UUID) | ID laporan yang ingin diambil |

**Response `200 OK`:**

```json
{
  "success": true,
  "message": "Public report retrieved",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Phishing link menyamar sebagai BCA",
    "description": "Link palsu tersebar menargetkan nasabah BCA melalui WhatsApp. Pengguna diminta memasukkan PIN ATM di halaman palsu.",
    "category": "phishing",
    "created_at": "2026-05-25 20:00:00",
    "updated_at": "2026-05-25 20:10:00",
    "images": [
      {
        "id": "img-uuid-1",
        "image_url": "https://storage.example.com/phishing-screenshot.jpg"
      }
    ]
  }
}
```

**Response `404 Not Found`:**

```json
{
  "success": false,
  "message": "Report not found"
}
```

**Field yang ditampilkan:**

| Field | Type | Keterangan |
|---|---|---|
| `id` | string (UUID) | ID laporan |
| `title` | string | Judul laporan |
| `description` | string | Deskripsi lengkap |
| `category` | string | `scam` \| `phishing` \| `judol` |
| `created_at` | string (datetime) | Waktu dibuat |
| `updated_at` | string (datetime) | Waktu terakhir diupdate |
| `images` | array | Maks. 3 gambar per laporan |

**Field yang TIDAK ditampilkan** (sengaja disembunyikan):
- `user_id` — tidak bisa trace laporan ke pemilik
- email, password, atau data user apapun

**cURL Example:**

```bash
curl http://localhost:3000/reports/public/550e8400-e29b-41d4-a716-446655440000
```

**JavaScript (fetch) Example:**

```javascript
const id = '550e8400-e29b-41d4-a716-446655440000';
const res = await fetch(`http://localhost:3000/reports/public/${id}`);
const json = await res.json();

if (!json.success) {
  console.error(json.message); // "Report not found"
} else {
  console.log(json.data); // report detail object
}
```

> **Penggunaan frontend:** Route detail halaman publik (`/reports/:id`) mengambil data dari endpoint ini tanpa perlu login.

---

## Link Checker API

### POST /link/check

Analyze a URL for safety using an external link checker service.

**Authorization:** `Bearer <token>` ✅ Required (must be logged in)

**Rate Limiting:** This endpoint calls an external API — handle errors gracefully on the client side.

**Request Body:**

```json
{
  "url": "https://suspicious-website.com"
}
```

**Validation Rules:**
- `url`: must be a valid URL format

**Response `200 OK`:**

```json
{
  "success": true,
  "message": "Link analyzed successfully",
  "data": {
    "url": "https://suspicious-website.com",
    "status": "safe",
    "score": 92
  }
}
```

**Possible `status` values** (from external API):

| Status | Description |
|---|---|
| `safe` | URL appears safe |
| `suspicious` | URL has suspicious characteristics |
| `malicious` | URL is known malicious |
| `unknown` | Could not determine safety |

**cURL Example:**

```bash
curl -X POST http://localhost:3000/link/check \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://suspicious-website.com"}'
```

**External API Integration:**  
The service sends a `POST` request to `LINK_CHECKER_API_URL` with:
- Header: `X-API-Key: <LINK_CHECKER_API_KEY>`
- Header: `Authorization: Bearer <LINK_CHECKER_API_KEY>`
- Body: `{"url": "<url>"}`
- Timeout: 15 seconds

> **Security:** The API key is **never** exposed in the response.

---

## Error Responses

### HTTP Status Codes

| Code | Meaning |
|---|---|
| `200` | Success |
| `201` | Created |
| `401` | Unauthorized — missing or invalid JWT |
| `403` | Forbidden — insufficient role permissions |
| `404` | Not Found |
| `409` | Conflict — duplicate username/email |
| `422` | Unprocessable Entity — validation failed |
| `500` | Internal Server Error |
| `502` | Bad Gateway — external API error |
| `504` | Gateway Timeout — external API timeout |
| `503` | Service Unavailable — link checker not configured |

### Common Error Examples

**Missing token:**
```json
{
  "success": false,
  "message": "Missing or invalid Authorization header"
}
```

**Expired/invalid token:**
```json
{
  "success": false,
  "message": "Invalid or expired token"
}
```

**Insufficient permissions:**
```json
{
  "success": false,
  "message": "You are not authorized to view this report"
}
```

**Validation error:**
```json
{
  "success": false,
  "message": "Password must be at least 8 characters"
}
```

**Duplicate email:**
```json
{
  "success": false,
  "message": "Email already registered"
}
```

**Not found:**
```json
{
  "success": false,
  "message": "Report not found"
}
```

---

## Authorization Summary

| Endpoint | Public | User | Admin |
|---|---|---|---|
| `POST /auth/register` | ✅ | ✅ | ✅ |
| `POST /auth/login` | ✅ | ✅ | ✅ |
| `GET /reports/public` | ✅ | ✅ | ✅ |
| `GET /reports/public/:id` | ✅ | ✅ | ✅ |
| `GET /auth/me` | ❌ | ✅ | ✅ |
| `GET /reports` | ❌ | ✅ (own) | ✅ (all) |
| `GET /reports/:id` | ❌ | ✅ (own) | ✅ (all) |
| `POST /reports` | ❌ | ✅ | ✅ |
| `PUT /reports/:id` | ❌ | ✅ (own) | ✅ (all) |
| `DELETE /reports/:id` | ❌ | ✅ (own) | ✅ (all) |
| `POST /link/check` | ❌ | ✅ | ✅ |

---

## Database Schema

```sql
-- Users table
CREATE TABLE users (
    id            VARCHAR(36) PRIMARY KEY,
    username      VARCHAR(50) NOT NULL UNIQUE,
    email         VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role          ENUM('admin', 'user') NOT NULL DEFAULT 'user',
    created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Reports table
CREATE TABLE reports (
    id          VARCHAR(36) PRIMARY KEY,
    user_id     VARCHAR(36) NOT NULL,
    title       VARCHAR(200) NOT NULL,
    description TEXT NOT NULL,
    category    ENUM('scam', 'phishing', 'judol') NOT NULL,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Report images table (max 3 per report)
CREATE TABLE report_images (
    id        VARCHAR(36) PRIMARY KEY,
    report_id VARCHAR(36) NOT NULL,
    image_url TEXT NOT NULL,
    FOREIGN KEY (report_id) REFERENCES reports(id) ON DELETE CASCADE
);
```

---

## Architecture

```
src/
├── main.rs          # Application entry, server setup, route wiring
├── state.rs         # AppState (DB pool, JWT secret, API keys)
├── routes/          # Route definitions (grouped by feature)
│   ├── auth.rs
│   ├── reports.rs
│   ├── link.rs
│   └── public.rs    # ← GET /reports/public (no auth)
├── handlers/        # Request handlers (business logic per endpoint)
│   ├── auth.rs
│   ├── reports.rs
│   ├── link.rs
│   └── public.rs    # ← list_public_reports handler
├── models/          # Database row structs (sqlx::FromRow)
│   ├── user.rs
│   └── report.rs
├── middleware/      # Axum middleware
│   ├── auth.rs      # JWT validation → injects Claims into extensions
│   └── role.rs      # Admin role guard
├── services/        # External integrations
│   └── link_checker.rs  # reqwest client for external link API
├── dto/             # Data Transfer Objects (request/response shapes)
│   ├── auth.rs
│   ├── report.rs
│   ├── link.rs
│   └── public.rs    # ← PublicReportResponse, PaginatedPublicReports
└── utils/           # Shared utilities
    ├── response.rs  # Consistent JSON response helpers
    ├── jwt.rs       # JWT generation & verification
    └── hash.rs      # Argon2 password hashing
```

---

*Generated for Rakshaka v0.1.0 — Built with Axum + SQLx + Argon2 + JWT*
