# My PC Kit

**Modern Windows System Utility · RAM Optimizer · Deep Cache Cleaner · Hardware Monitor · Power Controls**

<p align="center">
  <img src="src/assets/brand/logo.png" alt="My PC Kit — Windows System Utility" width="128" />
</p>

<p align="center">
  <strong>My PC Kit</strong> là bộ công cụ tối ưu hóa và dọn dẹp hệ thống Windows hiện đại, mượt mà và nhẹ nhàng, được xây dựng trên nền tảng <strong>Tauri 2 + Vue 3 + Rust + Tailwind CSS 4</strong>. Hỗ trợ đầy đủ song ngữ <strong>Tiếng Việt & English</strong> cùng giao diện sáng (Light Theme) tinh tế, sắc nét.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Platform-Windows_10%2F11-0078D6?style=for-the-badge&logo=windows&logoColor=white" alt="Windows Platform" />
  <img src="https://img.shields.io/badge/Tauri-2.0-FFC131?style=for-the-badge&logo=tauri&logoColor=black" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Vue-3.5-42b883?style=for-the-badge&logo=vuedotjs&logoColor=white" alt="Vue 3" />
  <img src="https://img.shields.io/badge/Rust-2021-DEA584?style=for-the-badge&logo=rust&logoColor=black" alt="Rust" />
  <img src="https://img.shields.io/badge/TypeScript-5-3178C6?style=for-the-badge&logo=typescript&logoColor=white" alt="TypeScript" />
  <img src="https://img.shields.io/badge/License-MIT-22c55e?style=for-the-badge" alt="License" />
</p>

---

## 🌟 Tính năng nổi bật (Features)

| Phân hệ | Chi tiết tính năng |
| :--- | :--- |
| 🧹 **Dọn dẹp nhanh (Quick Cleaner)** | Dọn sạch tệp tạm hệ thống (`%TEMP%`), Windows Prefetch và làm rỗng Thùng rác (Recycle Bin) chỉ với 1 cú nhấp. An toàn, tự động bỏ qua các file đang được sử dụng. |
| 🔍 **Dọn dẹp chuyên sâu (Deep Cleanup)** | Quét và phân loại chi tiết bộ nhớ đệm ứng dụng (Telegram, Discord, Steam, Spotify,...), trình duyệt (Chrome, Edge, Brave, Firefox, Opera) và cache công cụ lập trình (npm, pnpm, pip, cargo, go module, docker,...). Cho phép chọn lọc trước khi xóa. |
| ⚡ **Giải phóng RAM (Memory Cleaner)** | Tối ưu hóa các danh sách bộ nhớ Windows (Standby List, Working Set, System File Cache,...) bằng Win32 API thực thụ. Tích hợp tính năng **Tự động tối ưu định kỳ** hoặc khi RAM trống xuống dưới ngưỡng thiết lập. |
| 📊 **Giám sát thời gian thực (Live Monitor)** | Theo dõi trực quan tải CPU, RAM, ổ đĩa C và GPU. Tích hợp **Banner đánh giá sức khỏe hệ thống** và nút **"Tối ưu RAM ngay"** chỉ với 1 cú click tại trang chủ. |
| 🚀 **Thao tác nhanh (Quick Actions)** | Phím tắt mở nhanh 15 công cụ quản trị Windows phổ biến: Task Manager, Device Manager, Command Prompt (Admin), PowerShell (Admin), Disk Management, Registry Editor, Snipping Tool,... |
| 🔌 **Quản lý nguồn điện (Power Tools)** | Tắt máy, khởi động lại, ngủ (Sleep), khóa màn hình và tính năng **Hẹn giờ tắt máy (Scheduled Shutdown)** kèm đồng hồ đếm ngược trực quan. |
| 📋 **Thông tin phần cứng (System Info)** | Xuất báo cáo cấu hình chi tiết (CPU, GPU, RAM, Bo mạch chủ, BIOS, Pin/Nguồn, Màn hình, Ổ đĩa) và hỗ trợ sao chép vào khay nhớ tạm. |
| 🎨 **Giao diện hiện đại & Đa ngôn ngữ** | Giao diện màu sáng tinh khôi, hỗ trợ chuyển đổi nhanh Light / Dark ngay trên thanh tiêu đề. Hỗ trợ đầy đủ **Tiếng Việt** và **English**. |
| 🖥️ **Khay hệ thống (System Tray)** | Chạy ngầm nhẹ nhàng ở góc màn hình, tối ưu RAM tức thì từ menu chuột phải, không tốn tài nguyên máy. |

---

## 📸 Ảnh chụp màn hình (Screenshots)

<div align="center">
  <img src="images/screenshot1.png" alt="My PC Kit — System Monitor" width="85%" />
  <p><em>Trang Giám sát Hệ thống & Thao tác nhanh</em></p>

  <img src="images/screenshot2.png" alt="My PC Kit — Memory Cleaner" width="85%" />
  <p><em>Bộ công cụ Giải phóng RAM & Tối ưu hóa Bộ nhớ</em></p>

  <img src="images/screenshot3.png" alt="My PC Kit — Deep Cleaner" width="85%" />
  <p><em>Dọn dẹp chuyên sâu các bộ nhớ đệm hệ thống và ứng dụng</em></p>

  <img src="images/screenshot4.png" alt="My PC Kit — Hardware Information" width="85%" />
  <p><em>Báo cáo cấu hình và chi tiết phần cứng máy tính</em></p>
</div>

---

## 🛠️ Công nghệ cốt lõi (Tech Stack)

- **Application Shell:** [Tauri 2](https://tauri.app/) (Cửa sổ khởi động mượt mà, tối ưu dung lượng RAM và CPU).
- **Frontend:** [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) + [Pinia](https://pinia.vuejs.org/) + [Tailwind CSS 4](https://tailwindcss.com/).
- **Internationalization:** [vue-i18n](https://vue-i18n.intlify.dev/) (Song ngữ Tiếng Việt & English).
- **Backend Core:** Rust ([sysinfo](https://crates.io/crates/sysinfo), [windows-rs](https://crates.io/crates/windows) tương tác trực tiếp với Win32 / NT API).
- **Icon Set:** [Lucide Icons](https://lucide.dev/).

---

## 💻 Cài đặt & Phát triển (Development)

### Yêu cầu môi trường
- [Node.js](https://nodejs.org/) (phiên bản 18 trở lên)
- [pnpm](https://pnpm.io/) (khuyên dùng) hoặc `npm`
- [Rust & Cargo](https://www.rust-lang.org/tools/install) (dành cho phần backend desktop)
- Windows C++ Build Tools (MSVC)

### Các lệnh thực thi

```bash
# 1. Cài đặt các thư viện cần thiết
pnpm install

# 2. Khởi chạy bản xem trước giao diện web (Vite Hot Reload)
pnpm dev

# 3. Chạy ứng dụng desktop hoàn chỉnh với Tauri (Rust + Vue)
pnpm tauri:dev

# 4. Kiểm tra mã nguồn & type check
pnpm build

# 5. Đóng gói bộ cài đặt Windows (.exe / NSIS)
pnpm tauri:build
```

---

## 📦 Đóng gói xuất xưởng (Build Release)

Khi chạy `pnpm tauri:build`, file cài đặt sẽ được tạo tự động tại:
```
src-tauri/target/release/bundle/nsis/MyPCKit_X.X.X_x64-setup.exe
```
Cài đặt dưới quyền người dùng hiện tại (`currentUser`), không đòi hỏi quyền Admin khi cài đặt, hỗ trợ gỡ cài đặt sạch sẽ từ Start Menu hoặc Control Panel.

---

## 📄 Bản quyền (License)

Dự án được phát hành theo giấy phép **[MIT License](LICENSE)** — bạn hoàn toàn có thể sử dụng, tùy biến và phân phối tự do.

---

<p align="center">
  <strong>My PC Kit</strong> — <em>Tinh gọn, mạnh mẽ và thấu hiểu máy tính của bạn.</em>
</p>
