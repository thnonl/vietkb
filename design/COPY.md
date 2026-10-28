# VietKB — Copy (chữ trên giao diện), v1

**Người viết:** UX writer. **Phạm vi:** mọi chuỗi người dùng đọc trên 4 tab (Gõ · Gõ tắt · Phím tắt · Hệ
thống), menu ở biểu tượng VietKB, thông báo, trạng thái lỗi/rỗng, lần mở đầu tiên.
**Nguồn:** `docs/evkey-clone-work/20-spec.md` §A, §B.2, §C · `design/DESIGN-SPEC.md` · `design/light/*.html`
· `50-test-cases.md` §3. Nhãn EVKey đối chiếu thêm với các bài hướng dẫn EVKey (gearvn, cellphones,
quantrimang): `Bật kiểm tra chính tả`, `Tự động khôi phục phím với từ sai`, `Sửa lỗi dấu gợi ý`,
`Khởi động cùng Windows`, `Chạy với quyền Admin`, `Hỗ trợ ứng dụng Metro`,
`Sử dụng Clipboard để gửi phím`, `Vẫn cho phép gõ tắt khi tắt tiếng Việt`, `Loại trừ ứng dụng`, `Bảng gõ tắt`.

**Người đọc:** cô giáo, kế toán, sinh viên, nhân viên văn phòng 45 tuổi. Biết "gõ tiếng Việt", "Telex",
"Unicode", "UniKey". Không biết "hook", "IME", "charset", "encoding", "process", "khay hệ thống".

**Ngân sách độ dài (cứng)** — đếm theo ký tự NFC, kể cả dấu cách và dấu câu; cột *độ dài* ghi `n/ngân sách`:
nhãn ≤ 30 · mô tả ≤ 64 · nút ≤ 16 · tab ≤ 12 · toast ≤ 72 · lỗi ≤ 110 · mục menu khay ≤ 24 · gợi ý lần đầu ≤ 70.
Chuỗi có tham số được đếm với **giá trị mẫu dài nhất hợp lý** ghi trong chuỗi (ví dụ tên ứng dụng
`Visual Studio Code`, số `5.000`). `⚠ VƯỢT` = lỗi, phải sửa trước khi dùng.

`CẦN CHỐT` = chuỗi mô tả thứ spec chưa định nghĩa; chuỗi đưa ra là bản nháp, chưa được dùng cho tới khi chốt.

---

## §1 Thuật ngữ — một khái niệm, một từ, ở mọi nơi (kể cả thông báo lỗi)

| Khái niệm | Từ dùng | Từ không dùng | Vì sao |
|---|---|---|---|
| Bật/tắt bộ gõ | **bật/tắt tiếng Việt**; trạng thái: **Tiếng Việt** (bật) · **Đã tắt tiếng Việt** (tắt) | `English`, `tiếng Anh`, `chế độ E`, `chế độ V`, `IME` | Câu quen của EVKey/UniKey. `English` đụng với lựa chọn Ngôn ngữ hiển thị `English` và là chữ nước ngoài giữa giao diện Việt; khi tắt, người dùng không "gõ tiếng Anh" mà gõ chữ thường không dấu. (Spec C.0 ghi `Tiếng Việt / English` — đề xuất đổi.) |
| Telex / VNI / VIQR | **kiểu gõ** | `phương thức nhập`, `kiểu nhập`, `input method` | Từ EVKey/UniKey ai cũng biết. |
| Charset | **bảng mã** | `kiểu mã hoá`, `mã hoá`, `charset`, `encoding`, chữ `mã` đứng một mình | Từ EVKey/UniKey. Chữ `mã` đứng riêng bị hiểu thành "mã lệnh", "mã số" → không bao giờ viết `mã` thay cho `bảng mã`. |
| Tên các bảng mã | **Unicode dựng sẵn**, **Unicode tổ hợp**, **TCVN3 (ABC)**, **VNI Windows**, **VIQR**, **VISCII**, **BK HCM1**, **VPS**, **UTF-8 Literal** | `Unicode NFC/NFD`, `UTF-8` trơn | Tên UniKey/EVKey. `UTF-8` trơn bị hiểu là "Unicode bình thường", trong khi spec A11 là kiểu gửi từng byte → dùng tên UniKey `UTF-8 Literal` (CẦN CHỐT, xem §8). |
| òa/oà, úy/uý | **kiểu đặt dấu**: **Kiểu mới** (hòa, thúy) · **Kiểu cũ** (hoà, thuý) | `vị trí dấu thanh`, `dấu kiểu cũ (oà/uý)`, `old tone style` | Người Việt nói "bỏ dấu kiểu cũ/kiểu mới"; ví dụ trong nút là bằng chứng. |
| Hành động nhấn phím dấu | **gõ dấu** | `bỏ dấu`, `thêm dấu` (khi nói về thao tác) | `bỏ dấu` hai nghĩa: "đặt dấu" và "xoá dấu" (văn bản bỏ dấu). `đặt dấu` dành riêng cho *chỗ dấu nằm* (kiểu đặt dấu). |
| Dấu tiếng Việt nói chung | **dấu** | `dấu thanh`, `dấu phụ`, `diacritic` | Người dùng gọi chung mũ, móc, trăng, sắc, huyền… là "dấu". |
| Macro | **gõ tắt** (tính năng) · **từ tắt** (chữ gõ vào) · **nội dung** (chữ hiện ra) · **mục** (một cặp) | `macro`, `viết tắt`, `từ khoá`, `trigger`, `phím tắt` (cho từ tắt), `thay thế bởi`, `dòng`, `bản ghi` | `Gõ tắt` là từ EVKey/UniKey. `dòng` dành cho số dòng của nội dung (`+1 dòng`), không dùng để đếm mục. |
| Cửa sổ bảng macro đầy đủ | **bảng gõ tắt** | `bảng đầy đủ`, `danh sách macro` | Tên EVKey `Bảng gõ tắt`; trùng với mục menu khay. |
| Hotkey | **phím tắt**; chuỗi phím đang bấm: **tổ hợp phím** (gọn: **tổ hợp**) | `hotkey`, `phím nóng`, `chord`, `phím kết hợp` | Từ Windows tiếng Việt. Đọc gần `gõ tắt` nhưng cả hai đều đã quen; không đổi tên. |
| Nhấn phím / bấm chuột | **nhấn** (phím trên bàn phím) · **bấm** (chuột, nút trên màn hình) | `click`, `nhấp`, `ấn`, dùng lẫn trong một câu | Một câu như "Nhấn vào ô rồi bấm tổ hợp" làm người đọc phải đoán. |
| Danh sách app bị tắt | **loại trừ ứng dụng** | `danh sách đen`, `blacklist`, `bỏ qua ứng dụng`, `chặn` | Tên EVKey. Giữ nguyên dù trong danh sách có cả quy tắc "Bảng mã TCVN3" (không phải loại trừ thật) — đổi tên sẽ làm người dùng EVKey không tìm thấy. |
| Ứng dụng đang chạy | **ứng dụng đang mở** | `tiến trình`, `process`, `.exe` (trong câu) | Tên tệp `.exe` chỉ hiện ở dòng phụ khi cần phân biệt. |
| Reset về mặc định | **đặt lại** (`Đặt lại mặc định`, `Đặt lại tất cả`) | `khôi phục` (cho reset), `reset`, `thiết lập lại` | `khôi phục` đã là từ của *khôi phục từ gõ sai*; hai nghĩa trên cùng một màn hình Phím tắt là lỗi. `Đặt lại` là từ Windows tiếng Việt. |
| Đưa từ về phím gốc | **khôi phục từ gõ sai** | `hoàn nguyên`, `trả lại phím`, `restore` | Giữ từ khoá `khôi phục` của EVKey (`Tự động khôi phục phím với từ sai`) nhưng gọn hơn. |
| Undo trong toast | **Hoàn tác** | `Undo`, `Quay lại`, `Khôi phục` | Từ Windows/Office tiếng Việt. |
| Settings | **cài đặt** (cả cửa sổ, cả tệp: **tệp cài đặt**) · một mục riêng: **tuỳ chọn** | `cấu hình`, `thiết lập`, `config` | Một từ cho cả nút `Mở cài đặt`, lỗi `Chưa lưu được cài đặt`, tệp. `Tuỳ chỉnh` chỉ dùng cho chế độ quy tắc ứng dụng của spec C.5. |
| Import / export | **Nhập từ tệp… / Xuất ra tệp…**; trong toast: **đã nhập / đã xuất** | `Nhập…` trơn, `import`, `export`, `tải lên` | `Nhập` trơn cạnh nút `Thêm` đọc thành "gõ vào". |
| Tray | **biểu tượng VietKB (cạnh đồng hồ)**; menu: **menu ở biểu tượng** | `khay hệ thống`, `system tray`, `tray` | Người dùng không biết "khay hệ thống"; ai cũng thấy đồng hồ. |
| Restart app | **mở lại VietKB** | `khởi động lại` (trơn), `restart` | `Khởi động lại` bị hiểu là khởi động lại *máy* — người dùng sẽ ngại bấm. |
| Hook / engine | **bộ gõ** | `hook`, `engine`, `dịch vụ`, `tiến trình` | Nói việc xảy ra, không nói cơ chế. |
| Clipboard | **Clipboard** chỉ trong tên tuỳ chọn EVKey; trong câu: **chữ đã chép** | `bảng tạm`, `clipboard` viết thường giữa câu | Người dùng EVKey nhận ra chữ `Clipboard`; người còn lại hiểu "chữ đã chép". |
| Phím cách / Enter | trong câu: **dấu cách**, **xuống dòng**; trên phím vẽ: `Space`, `Enter` | `phím Space`, `khoảng trắng`, `spacebar` | Cách nói hằng ngày; phím vẽ giữ chữ in trên bàn phím. |
| Quyền admin | **quyền Admin** | `quyền quản trị viên`, `elevated` | Từ EVKey `Chạy với quyền Admin`; ngắn hơn. |
| Tương thích | **Sửa lỗi dấu gợi ý**, **Hỗ trợ ứng dụng Metro** | — | Giữ nguyên nhãn EVKey; dòng mô tả giải thích bằng lời thường. |
| Ký tự | **ký tự** | `kí tự`, `char` | Một cách viết. |
| Số lớn | **5.000**, **1.234** (dấu chấm ngăn nghìn) | `5000`, `1 234`, `1,234` | Cách viết số của người Việt (spec C.4 đang ghi `1 234` → đổi). |
| Chính tả của chính giao diện | dấu theo **kiểu mới**: `tuỳ`, `huỷ`, `xoá`, `khoá`, `thuý` | `tùy`, `hủy`, `xóa` | Khớp mặc định của sản phẩm (Kiểu mới). Không trộn hai kiểu trong cùng giao diện. |

---

## §2 Giọng văn — 5 quy tắc

| # | Quy tắc | Sai → Đúng |
|---|---|---|
| 1 | **Giải thích bằng ví dụ, không bằng luật.** Một cặp "gõ → ra" đáng giá hơn một câu ngữ pháp. | `Cho phép gõ dấu sau phụ âm cuối` → `hoangf → hoàng: gõ dấu sau cả từ cũng được` |
| 2 | **Nói điều người dùng thấy, không nói máy làm gì bên trong.** Không `hook`, `gửi phím`, `process`, `mã hoá`. | `Lỗi hook — Khởi động lại` → `Bộ gõ đang không chạy nên chưa gõ được tiếng Việt · Chạy lại` |
| 3 | **Nói thẳng với người dùng.** Câu mệnh lệnh nhẹ hoặc "bạn" khi cần; không `vui lòng`, `quý khách`, `người dùng`. | `Quý khách vui lòng khởi động lại ứng dụng để áp dụng` → `Cần mở lại VietKB · Mở lại ngay` |
| 4 | **Nói khẳng định, không phủ định kép.** Nói điều *có* xảy ra. | `Không đặt dấu vào những từ không phải tiếng Việt` → `Chỉ thêm dấu khi ra từ tiếng Việt: search vẫn là search` |
| 5 | **Lỗi: bình tĩnh, không đổ lỗi, có bước tiếp.** Việc gì đã xảy ra + cái gì vẫn ổn + một nút. | `Bạn đã chọn sai định dạng tệp!` → `Không đọc được tệp gotat.txt: dòng 12 không đúng dạng. Chưa nhập mục nào.` |

Thêm (không tính là quy tắc, là kỷ luật): không VIẾT HOA TOÀN BỘ (dấu chồng bị cắt); không dấu `!`;
không dấu `…` ở cuối mô tả (cắt chữ là lỗi, viết lại cho ngắn); dấu `…` chỉ ở nút/mục menu mở thêm một cửa sổ.

---

## §3 Chữ từng màn hình — sẵn để dán

### 3.0 Khung cửa sổ (chung cho 4 tab)

| vị trí | chuỗi đề xuất | độ dài | ghi chú |
|---|---|---|---|
| Tiêu đề cửa sổ (Alt+Tab, thanh tác vụ) | `Cài đặt VietKB` | 14/30 | Cửa sổ không viền nên tiêu đề chỉ hiện ở Alt+Tab; nói rõ đây là gì. |
| Tab 1 | `Gõ` | 2/12 | giữ |
| Tab 2 | `Gõ tắt` | 6/12 | số mục đi kèm, viết `148`, `5.000` |
| Tab 3 | `Phím tắt` | 8/12 | giữ |
| Tab 4 | `Hệ thống` | 8/12 | giữ |
| Nút thu nhỏ (tooltip) | `Thu nhỏ` | 7/16 | |
| Nút đóng (tooltip) | `Đóng, VietKB vẫn chạy` | 21/64 | đóng = ẩn xuống biểu tượng; nói ngay để người dùng khỏi lo |
| Chip trạng thái (tab 2–4), đang bật | `Tiếng Việt · Telex · Unicode` | 28/30 | `Unicode` = tên ngắn của Unicode dựng sẵn |
| Chip trạng thái, đang tắt | `Đã tắt tiếng Việt` | 17/30 | chấm xám thay chấm tím |
| Chip, ứng dụng hiện tại bị loại trừ | `Đang ở Chrome · đã loại trừ` | 27/30 | tên hiển thị của ứng dụng, cắt ở 12 ký tự |
| Chip (tooltip) | `Bấm để tắt tiếng Việt` | 21/64 | khi tắt: `Bấm để bật tiếng Việt` |
| Tên ngắn bảng mã trong chip / Thử gõ | `Unicode` · `Unicode tổ hợp` · `TCVN3` · `VNI Windows` · `VIQR` · `VISCII` | — | giữ chip ≤ 30 |

### 3.1 Tab Gõ

| vị trí | chuỗi đề xuất | độ dài | ghi chú |
|---|---|---|---|
| Hàng trạng thái — nhãn công tắc | `Tiếng Việt` | 10/30 | công tắc nói trạng thái; chữ không đổi khi tắt |
| Hàng trạng thái — tooltip công tắc | `Bật/tắt tiếng Việt (Ctrl + Shift)` | 33/64 | phím lấy theo cài đặt hiện tại |
| Hàng trạng thái — nhãn ô chọn 1 | `Kiểu gõ` | 7/30 | |
| Kiểu gõ — lựa chọn | `Telex` · `VNI` · `VIQR` | — | giữ |
| Hàng trạng thái — nhãn ô chọn 2 | `Bảng mã` | 7/30 | |
| Bảng mã — mục 1 | `Unicode dựng sẵn` | 16/30 | |
| Bảng mã — mô tả mục 1 (dòng phụ trong danh sách xổ) | `Dùng cho hầu hết ứng dụng: Word, Zalo, trình duyệt` | 50/64 | nếu danh sách xổ có dòng phụ; không thì bỏ |
| Bảng mã — mục 2 | `Unicode tổ hợp` | 14/30 | |
| Bảng mã — mô tả mục 2 | `Chỉ chọn khi phần mềm yêu cầu` | 29/64 | |
| Bảng mã — mục 3 | `TCVN3 (ABC)` | 11/30 | |
| Bảng mã — mô tả mục 3 | `Văn bản cũ dùng phông .VnTime, .VnArial` | 39/64 | |
| Bảng mã — mục 4 | `VNI Windows` | 11/30 | |
| Bảng mã — mô tả mục 4 | `Văn bản cũ dùng phông VNI-Times` | 31/64 | |
| Bảng mã — mục 5 | `VIQR` | 4/30 | |
| Bảng mã — mô tả mục 5 | `Dấu viết bằng ký hiệu: việt thành vie^.t` | 40/64 | |
| Bảng mã — mục 6 | `VISCII` | 6/30 | |
| Bảng mã — mô tả mục 6 | `Văn bản cũ dùng phông VISCII` | 28/64 | |
| Bảng mã — mục 7, 8 | `BK HCM1` · `VPS` | — | CẦN CHỐT: spec A11 để ngoài Sprint 1 — ẩn hẳn hay hiện mờ? Nếu hiện mờ: mô tả `Chưa hỗ trợ trong bản này` |
| Bảng mã — mục 9 | `UTF-8 Literal` | 13/30 | CẦN CHỐT (§8) |
| **Nhóm Dấu** | | | |
| Kiểu đặt dấu — nhãn | `Kiểu đặt dấu` | 12/30 | giữ |
| Kiểu đặt dấu — mô tả | `Hai kiểu chỉ khác nhau ở vần oa, oe, uy` | 39/64 | nói hậu quả (khác ít) thay vì thuật ngữ "dấu thanh" |
| Kiểu đặt dấu — nút 1 | `Kiểu mới · hòa, thúy` | 20/30 | giữ; mặc định |
| Kiểu đặt dấu — nút 2 | `Kiểu cũ · hoà, thuý` | 19/30 | giữ |
| Gõ dấu ở cuối từ — nhãn | `Gõ dấu ở cuối từ` | 16/30 | thay `Bỏ dấu…` (§1) |
| Gõ dấu ở cuối từ — mô tả | `hoangf → hoàng: gõ dấu sau cả từ cũng được` | 42/64 | `hoangf` dạng phím (mono) |
| Tự sửa ưo — nhãn | `Tự sửa ưo thành ươ` | 18/30 | giữ |
| Tự sửa ưo — mô tả | `nguwoif → người, chỉ cần một w` | 30/64 | giữ (đúng spec A1 auto-horn) |
| **Nhóm Chính tả và khôi phục** | | | |
| Kiểm tra chính tả — nhãn | `Kiểm tra chính tả` | 17/30 | nhãn EVKey bỏ chữ `Bật` (công tắc đã nói) |
| Kiểm tra chính tả — mô tả | `Chỉ thêm dấu khi ra từ tiếng Việt: search vẫn là search` | 55/64 | bỏ ví dụ `excel` — xem §7 |
| Tự động khôi phục — nhãn | `Tự động khôi phục từ gõ sai` | 27/30 | giữ |
| Tự động khôi phục — mô tả | `gôgle tự trở về google khi bạn gõ dấu cách` | 42/64 | **bỏ trạng thái mờ "Cần bật Kiểm tra chính tả"** — spec A10/C.2: độc lập |
| Giữ nguyên email, web — nhãn | `Giữ nguyên email, địa chỉ web` | 29/30 | thay `Bảo vệ đường dẫn và mã` |
| Giữ nguyên email, web — mô tả | `ten@congty.vn, abc.com/xyz gõ sao ra vậy, không thêm dấu` | 56/64 | cũng áp dụng cho `a_b`, `x=y` (spec A9) — đủ đúng cho người thường |
| Viết hoa đầu câu — nhãn | `Viết hoa đầu câu` | 16/30 | thay `Viết hoa sau dấu câu` (vì có cả sau xuống dòng) |
| Viết hoa đầu câu — mô tả | `chào. tôi → chào. Tôi, cả sau ! ? và khi xuống dòng` | 51/64 | |
| **Nhóm Gõ tắt** | | | |
| Gõ tắt — nhãn | `Gõ tắt` | 6/30 | giữ |
| Gõ tắt — mô tả | `Gõ nth rồi dấu cách → Nguyễn Thu Hà` | 35/64 | lấy từ tắt đầu tiên của người dùng; khi bảng rỗng: `Gõ dc rồi dấu cách → được` |
| Gõ tắt — liên kết | `148 mục` | 7/16 | tooltip `Mở tab Gõ tắt` |
| Dòng bị mờ vì phụ thuộc (mẫu) | `Cần bật Gõ tắt · Bật` | 20/64 | mẫu `Cần bật <tên tuỳ chọn> · Bật` — chỉ cho phụ thuộc có thật (vd. Viết hoa theo từ tắt) |
| **Nâng cao** | | | |
| Dòng mở — nhãn | `Nâng cao` | 8/30 | giữ |
| Dòng mở — thẻ đếm | `8 tuỳ chọn` | 10/16 | giữ |
| Dòng mở — xem trước | `f j w z làm phụ âm, sửa lỗi dấu gợi ý, Metro, Clipboard` | 55/64 | không `…` cuối; tên EVKey để người quen nhận ra |
| Dòng quay lại | `‹ Tuỳ chọn chính` | 16/30 | giữ |
| f j w z — nhãn | `Cho phép f j w z làm phụ âm` | 27/30 | gần nhãn EVKey |
| f j w z — mô tả | `Gõ w ra w, không thành ư; hợp khi hay gõ tên nước ngoài` | 55/64 | spec AC-5 |
| Gõ tắt khi tắt — nhãn | `Gõ tắt cả khi tắt tiếng Việt` | 28/30 | nhãn EVKey `Vẫn cho phép gõ tắt khi tắt tiếng Việt` dài 38 → rút |
| Gõ tắt khi tắt — mô tả | `Đã tắt tiếng Việt, gõ nth vẫn ra Nguyễn Thu Hà` | 46/64 | |
| Hoa/thường theo từ tắt — nhãn | `Viết hoa theo từ tắt` | 20/30 | spec `macro_case_follow` |
| Hoa/thường theo từ tắt — mô tả | `dc → được, Dc → Được, DC → ĐƯỢC` | 31/64 | |
| Bàn phím không phải US — nhãn | `Tắt khi đổi bàn phím khác` | 25/30 | **CẦN CHỐT**: tuỳ chọn có trong DESIGN-SPEC nhưng không có trong spec B.2 |
| Bàn phím không phải US — mô tả | `Tự tắt khi Windows dùng bàn phím không phải loại Mỹ (US)` | 56/64 | CẦN CHỐT |
| Sửa lỗi dấu gợi ý — nhãn | `Sửa lỗi dấu gợi ý` | 17/30 | nhãn EVKey, giữ |
| Sửa lỗi dấu gợi ý — mô tả | `Chữ bị lặp ở ô có gợi ý như Chrome, Excel? Thử cách 1 hoặc 2` | 60/64 | |
| Sửa lỗi dấu gợi ý — lựa chọn | `Tắt` · `Cách 1` · `Cách 2` | — | spec `off/fix1/fix2`; EVKey cũng có "1" và "2" |
| Metro — nhãn | `Hỗ trợ ứng dụng Metro` | 21/30 | nhãn EVKey, giữ |
| Metro — mô tả | `Gõ được trong ứng dụng tải từ Microsoft Store, như Mail` | 55/64 | giải thích chữ "Metro" bằng thứ người dùng thấy |
| Clipboard — nhãn | `Gửi chữ qua Clipboard` | 21/30 | giữ chữ "Clipboard" của EVKey, bỏ "gửi phím" |
| Clipboard — mô tả | `Dùng khi ứng dụng bị mất chữ; chữ đã chép trước đó vẫn còn` | 58/64 | spec AC-65: Clipboard cũ được trả lại |
| Độ trễ — nhãn | `Độ trễ giữa các phím` | 20/30 | |
| Độ trễ — mô tả | `Tăng lên nếu ứng dụng chạy chậm bị lộn hoặc mất chữ` | 51/64 | **CẦN CHỐT** đơn vị và mặc định: spec `send_delay_us` = 0 µs, thiết kế ghi `8` không đơn vị |
| **Thử gõ** | | | |
| Nhãn | `Thử gõ` | 6/30 | giữ |
| Chữ mờ khi ô trống | `Gõ thử ở đây, chữ không được lưu lại` | 36/64 | nói luôn về riêng tư (spec C5/AC-85) |
| Dòng nhắc bên phải | `Telex · Unicode` | 15/30 | giữ; lấy theo cài đặt hiện tại |
| **Khác** | | | |
| Chữ xác nhận đã lưu (cạnh dòng) | `Đã lưu` | 6/16 | giữ (spec C.0) |

### 3.2 Tab Gõ tắt

| vị trí | chuỗi đề xuất | độ dài | ghi chú |
|---|---|---|---|
| Ô tìm — chữ mờ | `Tìm từ tắt hoặc nội dung` | 24/30 | giữ; phím vẽ `Ctrl` `F` |
| Nút nhập | `Nhập từ tệp…` | 12/16 | `Nhập…` trơn bị đọc là "gõ vào" |
| Nút xuất | `Xuất ra tệp…` | 12/16 | đi đôi với nút nhập |
| Nút chính | `Thêm` | 4/16 | giữ |
| Tiêu đề cột 1 | `Từ tắt` | 6/16 | |
| Tiêu đề cột 2 | `Nội dung` | 8/16 | |
| Ô chọn tất cả (tooltip) | `Chọn tất cả` | 11/16 | |
| Hàng thêm mới — ô 1 chữ mờ | `ví dụ: dc` | 9/16 | cột 112 px |
| Hàng thêm mới — ô 2 chữ mờ | `ví dụ: được · Enter để lưu` | 26/30 | |
| Thanh chọn — đếm | `Đã chọn 1 trong 148` | 19/30 | giữ; `Đã chọn 12 trong 5.000` vẫn ≤ 30 |
| Thanh chọn — nút | `Sửa` · `Xuất…` · `Xoá` | — | `Xuất…` có `…` vì mở hộp chọn nơi lưu; `Sửa` mờ khi chọn > 1 |
| Thanh chọn — bỏ chọn | `Bỏ chọn` | 7/16 | kèm phím `Esc` |
| Nút trên hàng (tooltip) | `Sửa` · `Tuỳ chọn khác` | — | menu `…`: CẦN CHỐT nội dung (spec chưa định nghĩa) |
| Thẻ nội dung nhiều dòng | `+1 dòng` | 7/16 | giữ |
| Chân bảng | `Đang hiện 8 trong 148 mục · gõ vào ô tìm để lọc` | 47/64 | |
| Chân bảng — liên kết | `Mở bảng gõ tắt` | 14/16 | tên EVKey, trùng mục menu khay `Bảng gõ tắt…` |
| Gõ tắt đang tắt (dòng trên bảng) | `Gõ tắt đang tắt, các mục chưa có tác dụng · Bật` | 47/64 | bảng mờ nhưng vẫn sửa được (spec C.4) |

### 3.3 Tab Phím tắt

| vị trí | chuỗi đề xuất | độ dài | ghi chú |
|---|---|---|---|
| Dòng hướng dẫn | `Bấm vào ô, nhấn tổ hợp mới. Esc để huỷ, Backspace để xoá.` | 57/64 | `Esc`, `Backspace` vẽ thành phím |
| Nhóm 1 | `Chuyển chế độ` | 13/30 | giữ |
| Bật/tắt — nhãn | `Bật/tắt tiếng Việt` | 18/30 | giữ |
| Bật/tắt — mô tả (tổ hợp chỉ có Ctrl/Shift/Alt) | `Ctrl + Shift + T trong trình duyệt vẫn mở lại tab` | 49/64 | giữ — chứng minh "nhả mới chuyển" bằng ví dụ |
| Bật/tắt — mô tả (tổ hợp có phím thường, vd. Alt + Z) | `Nhấn một lần để bật, nhấn lần nữa để tắt` | 40/64 | |
| Thẻ trong ô | `Nhả để chuyển` | 13/16 | giữ |
| Nút mũi tên mẫu (tooltip) | `Chọn nhanh` | 10/16 | |
| Đổi bảng mã — nhãn | `Đổi bảng mã` | 11/30 | giữ |
| Đổi bảng mã — mô tả | `Unicode → TCVN3 → VNI Windows` | 29/64 | **CẦN CHỐT** vòng đổi gồm những bảng mã nào, theo thứ tự nào |
| Đổi kiểu gõ — nhãn | `Đổi kiểu gõ` | 11/30 | giữ |
| Đổi kiểu gõ — mô tả | `Telex → VNI → VIQR` | 18/64 | giữ (giả định vòng 3 kiểu gõ, theo spec B) |
| Ô đang nghe phím — chưa nhấn gì | `Nhấn tổ hợp phím…` | 17/30 | spec C.6 bước 2 |
| Ô đang nghe phím — gợi ý cuối ô | `Esc huỷ` | 7/16 | giữ |
| Nút xác nhận / bỏ (tooltip) | `Dùng tổ hợp này` · `Huỷ` | — | spec C.6 bước 4 (✓ / ✕) |
| Dòng mẫu — dẫn | `Chọn nhanh:` | 11/30 | thay `Mẫu cho Bật/tắt:` |
| Mẫu 1 — chú thích | `mặc định` | 8/16 | giữ |
| Mẫu 2 — chú thích | `như UniKey` | 10/16 | giữ |
| Mẫu 4 — chữ trên phím | `Ctrl phải` | 9/16 | thay `Right Ctrl`; spec có 5 mẫu (thêm `Ctrl + Alt + Space`), thiết kế 4 |
| Nhóm 2 | `Công cụ` | 7/30 | giữ |
| Khôi phục từ — nhãn | `Khôi phục từ gõ sai` | 19/30 | giữ |
| Khôi phục từ — mô tả | `Đưa từ vừa gõ về đúng phím đã nhấn: gôgle → google` | 50/64 | |
| Khôi phục từ — cảnh báo (dòng 3) | `Cũng là phím Làm lại ở nhiều ứng dụng; không có từ để khôi phục thì vẫn Làm lại` | 79/110 | dòng này chạy hết bề ngang hàng, chứa được ~ 80 ký tự ở 12 px → giới hạn thực 80 |
| Thẻ cảnh báo vàng | `Dễ trùng` | 8/16 | thay `Trùng`; `Trùng` dành cho xung đột chặn (§4) |
| Mở cài đặt — nhãn | `Mở cài đặt` | 10/30 | giữ |
| Mở cài đặt — mô tả (chưa đổi) | `Mở cửa sổ này từ bất cứ đâu` | 27/64 | |
| Mở cài đặt — mô tả (đã đổi) | `Mặc định là Ctrl + Alt + S` | 26/64 | thay `Đã đổi từ…` |
| Nút đặt lại một dòng (tooltip) | `Đặt lại mặc định` | 16/16 | |
| Chuyển mã — nhãn | `Chuyển mã chữ đã chép` | 21/30 | thay `Chuyển mã clipboard` |
| Chuyển mã — mô tả | `Đổi chữ đã chép sang bảng mã đang dùng, rồi dán như thường` | 58/64 | **CẦN CHỐT**: đổi *từ* bảng mã nào? (spec chỉ ghi Sprint 2) |
| Ô trống | `Chưa đặt` | 8/16 | giữ |
| Ô trống — liên kết | `Đặt phím` | 8/16 | giữ |
| Chân — nút | `Đặt lại tất cả` | 14/16 | thay `Khôi phục tất cả` |
| Chân — dẫn trước 3 tổ hợp | `Windows đã dùng:` | 16/16 | thay `Không gán được:`; nói lý do |

### 3.4 Tab Hệ thống

| vị trí | chuỗi đề xuất | độ dài | ghi chú |
|---|---|---|---|
| Mục 1 | `Khởi động` | 9/16 | giữ (cột 120 px) |
| Khởi động cùng Windows — nhãn | `Khởi động cùng Windows` | 22/30 | nhãn EVKey, giữ |
| Khởi động cùng Windows — mô tả | `Bật máy là gõ được ngay, biểu tượng nằm cạnh đồng hồ` | 52/64 | bỏ "14 MB bộ nhớ" |
| Quyền Admin — nhãn | `Chạy với quyền Admin` | 20/30 | nhãn EVKey, giữ |
| Quyền Admin — mô tả (tắt) | `Cần khi gõ vào ứng dụng cũng chạy quyền Admin` | 45/64 | nói khi nào cần bật |
| Quyền Admin — mô tả (vừa bật, chờ mở lại) | `Cần mở lại VietKB, Windows sẽ hỏi xác nhận · Mở lại ngay` | 56/64 | `Mở lại ngay` là liên kết đỏ |
| Mục 2 | `Giao diện` | 9/16 | giữ |
| Âm thanh — nhãn | `Âm thanh khi bật/tắt` | 20/30 | giữ |
| Âm thanh — mô tả | `Kêu “tách” nhỏ mỗi lần bật hoặc tắt tiếng Việt` | 46/64 | |
| Ngôn ngữ — nhãn | `Ngôn ngữ hiển thị` | 17/30 | `Ngôn ngữ` trơn bị hiểu là ngôn ngữ gõ |
| Ngôn ngữ — mô tả | `Chữ trên cửa sổ và menu của VietKB, không đổi cách gõ` | 53/64 | |
| Ngôn ngữ — lựa chọn | `Tiếng Việt` · `English` | — | giữ; tên ngôn ngữ viết bằng chính ngôn ngữ đó |
| Màu nền — nhãn | `Màu nền` | 7/30 | thay dòng `Giao diện` trùng tên mục |
| Màu nền — mô tả | `Theo Windows: tự đổi khi Windows chuyển sáng hoặc tối` | 53/64 | |
| Màu nền — lựa chọn | `Theo Windows` · `Sáng` · `Tối` | — | `Theo hệ thống` → `Theo Windows` |
| Mục 3 | `Cập nhật` | 8/16 | giữ |
| Tự kiểm tra cập nhật — nhãn | `Tự kiểm tra cập nhật` | 20/30 | giữ |
| Tự kiểm tra cập nhật — mô tả | `Đang dùng bản 0.1.0 · chỉ xem có bản mới, không gửi gì khác` | 59/64 | |
| Nút | `Kiểm tra ngay` | 13/16 | giữ |
| Tệp cài đặt — nhãn | `Tệp cài đặt` | 11/30 | thay `Tệp cấu hình` |
| Tệp cài đặt — mô tả | `%APPDATA%\VietKB\config.toml` | 28/64 | giữ (đường dẫn, phông mono) |
| Nút | `Mở thư mục` | 10/16 | giữ |
| Nút nguy hiểm | `Đặt lại tất cả…` | 15/16 | thay `Khôi phục…`; xác nhận tại chỗ (§4) |
| Mục 4 | `Loại trừ ứng dụng` | 17/30 | tên EVKey, giữ |
| Mục 4 — dòng phụ | `6 ứng dụng` | 10/16 | giữ |
| Mục 4 — liên kết | `Quản lý` | 7/16 | giữ |
| Chế độ quy tắc | `Tắt tiếng Việt` · `Bật tiếng Việt` · `Bảng mã TCVN3` · `Kiểu gõ VNI` · `Tuỳ chỉnh` | — | spec C.5; khi ghi đè 1 thứ thì ghi rõ thứ đó |
| Danh sách rỗng — dòng chính | `Chưa loại trừ ứng dụng nào` | 26/30 | |
| Danh sách rỗng — dòng phụ | `Hay dùng cho trò chơi và cửa sổ lệnh` | 36/64 | spec C.5 gợi ý |

---

## §4 Trạng thái

Giọng mặc định: bình tĩnh, không đổ lỗi, có một bước tiếp. Toast có nút thì nút tính riêng (≤ 16).

| Trạng thái | Chuỗi | độ dài | Ở đâu | Giọng / ghi chú |
|---|---|---|---|---|
| 0 mục gõ tắt — tiêu đề | `Chưa có gõ tắt nào` | 18/30 | giữa bảng, tab Gõ tắt | trung tính |
| 0 mục — mô tả | `Gõ dc rồi dấu cách sẽ ra “được”. Thêm vài mục mẫu để thử:` | 57/64 | dưới tiêu đề | ví dụ trước, mời làm |
| 0 mục — 3 mục mẫu | `dc → được` · `ko → không` · `vn → Việt Nam` | — | 3 hàng bấm-để-thêm | spec C.4 |
| 0 mục — nút | `Thêm cả 3 mục` | 13/16 | dưới 3 mục mẫu | cạnh `Nhập từ tệp…` cho người chuyển từ EVKey |
| 0 mục — tab | `Gõ tắt` | 6/12 | tab | ẩn số khi bằng 0 |
| 1 mục — chân bảng | `Có 1 mục` | 8/64 | chân bảng | không `Đang hiện 1 trong 1` |
| 5.000 mục — tab | `Gõ tắt` + số `5.000` | — | tab | số dùng dấu chấm; tối đa `10.000` |
| 5.000 mục — chân bảng | `Đang hiện 8 trong 5.000 mục · gõ vào ô tìm để lọc` | 49/64 | chân bảng | |
| Tìm không thấy | `Không có mục nào khớp “ctdd”` | 28/64 | giữa bảng | |
| Tìm không thấy — nút | `Thêm “ctdd”` | 11/16 | dưới dòng trên | điền sẵn từ tắt; từ tìm > 9 ký tự thì nút ghi `Thêm mục này` |
| Đang nhập tệp | `Đang nhập… 2.300 / 5.000 mục` | 28/72 | thanh chân bảng | spec C.0: chỉ import có khung chờ |
| Đang tìm ứng dụng đang mở | `Đang tìm ứng dụng đang mở…` | 26/64 | hộp chọn ứng dụng | khung chờ |
| Lưu thất bại — toast | `Chưa lưu được cài đặt: tệp chỉ đọc. Vẫn dùng được đến khi thoát.` | 64/72 | toast, góc dưới phải | nói cái vẫn ổn; nút `Thử lại` |
| Lưu thất bại — lý do (thay `tệp chỉ đọc`) | `ổ đĩa đã đầy` · `không có quyền ghi` · `tệp đang được mở ở nơi khác` | — | trong toast | lý do dài nhất vẫn phải ≤ 72 cả câu — xem dòng dưới |
| Lưu thất bại — lý do dài nhất | `Chưa lưu được cài đặt: tệp đang được mở ở nơi khác. Vẫn dùng được.` | 66/72 | toast | bỏ "đến khi thoát" khi lý do dài |
| Lưu thất bại — chấm cảnh báo (tooltip) | `Chưa lưu vào tệp · Thử lại` | 26/64 | cạnh dòng vừa đổi | spec C.0 |
| Phím tắt trùng dòng khác | `Trùng với “Đổi kiểu gõ”` | 23/110 | dưới ô phím, chặn lưu | nút `Đổi chỗ` (thay `Hoán đổi`) |
| Phím tắt trùng — thẻ | `Trùng` | 5/16 | trong ô | chỉ dùng cho xung đột chặn |
| Trùng phím quen của ứng dụng (cảnh báo vàng, mẫu) | `Ctrl + C là phím Sao chép ở hầu hết ứng dụng. Vẫn đặt được nếu bạn muốn.` | 72/110 | dòng 3 của hàng | CẦN CHỐT: VietKB có "nuốt" phím đó không? Nếu có, thêm `Ứng dụng sẽ không nhận phím này nữa.` |
| — tên việc cho từng tổ hợp | Ctrl+Z `Hoàn tác` · Ctrl+Y, Ctrl+Shift+Z `Làm lại` · Ctrl+C `Sao chép` · Ctrl+V `Dán` · Ctrl+X `Cắt` · Ctrl+A `Chọn tất cả` · Ctrl+S `Lưu` · Ctrl+F `Tìm` · Ctrl+Shift+T `Mở lại tab` · Alt+Shift, Win+Space `Đổi bàn phím của Windows` · Ctrl+Space `Bộ gõ tiếng Trung, Nhật` | — | điền vào mẫu trên | tên lệnh theo Windows/Office tiếng Việt |
| Windows giữ riêng (Win+L…) | `Windows đã dùng Win + L để khoá máy. Hãy nhấn tổ hợp khác.` | 58/110 | dưới ô, ô vẫn nghe phím | Alt+Tab `để chuyển cửa sổ` · Ctrl+Alt+Del `để mở màn hình bảo mật` · Win+… `cho phím Windows` |
| Một phím thường không kèm gì | `Cần kèm Ctrl, Alt hoặc Win, vì phím A một mình sẽ gõ ra chữ.` | 60/110 | dưới ô, ô vẫn nghe phím | giải thích lý do, không trách |
| Shift + chữ | `Shift + A dùng để gõ chữ hoa. Hãy kèm Ctrl, Alt hoặc Win.` | 57/110 | dưới ô | |
| Xoá phím bật/tắt (không cho trống) | `Bật/tắt tiếng Việt cần một phím tắt, đã giữ lại phím cũ.` | 56/110 | dưới ô | thay `Phím bật/tắt không được để trống` |
| Đặt lại một dòng | `Đã đặt lại phím Mở cài đặt` | 26/72 | toast + `Hoàn tác` | 5 s |
| Ứng dụng đã có quy tắc | `Chrome đã có trong danh sách loại trừ, đang được tô sáng.` | 57/110 | hộp chọn ứng dụng; dòng cũ được chọn | spec C.5 |
| Không xem được đường dẫn ứng dụng (tooltip) | `Không xem được vị trí tệp của ứng dụng này` | 42/64 | dòng ứng dụng | chỉ hiện tên |
| Từ tắt trống | `Nhập từ tắt, ví dụ dc` | 21/110 | dưới ô từ tắt, không lưu | |
| Từ tắt quá dài | `Từ tắt dài tối đa 32 ký tự, đang có 40` | 38/110 | dưới ô từ tắt | số thật |
| Từ tắt có dấu cách | `Từ tắt viết liền, không có dấu cách` | 35/110 | dưới ô từ tắt | |
| Từ tắt trùng | `“dc” đã có, đang ra “được”.` | 27/110 | dưới ô từ tắt | nút `Mở mục đó` (10) |
| Nội dung quá dài | `Nội dung dài tối đa 4.000 ký tự, đang có 4.250` | 46/110 | dưới ô nội dung | |
| Nội dung trống | `Nhập nội dung sẽ hiện ra khi gõ từ tắt` | 38/110 | dưới ô nội dung | CẦN CHỐT: spec C.4 không nói nội dung trống có hợp lệ không |
| Tệp nhập sai dạng | `Không đọc được tệp gotat.txt: dòng 12 không đúng dạng. Chưa nhập mục nào.` | 73/110 | toast/khung lỗi, tab Gõ tắt | tất cả hoặc không (UI-15); CẦN CHỐT dạng tệp |
| — dòng hướng dẫn kèm | `Mỗi dòng gồm từ tắt, một dấu Tab, rồi nội dung` | 46/110 | dưới lỗi trên | CẦN CHỐT: phụ thuộc dạng tệp đã chốt |
| Nhập xong | `Đã nhập 1.234 mục, bỏ qua 3 mục trùng` | 37/72 | toast | spec C.4 (số dùng dấu chấm) |
| Xuất xong | `Đã xuất 148 mục ra gotat.txt` | 28/72 | toast + nút `Mở thư mục` | |
| Xoá mục | `Đã xoá 3 mục gõ tắt` | 19/72 | toast + `Hoàn tác` | 5 s |
| Đặt lại tất cả — hỏi tại chỗ | `Đặt lại mọi cài đặt về mặc định?` | 32/110 | thay dòng Tệp cài đặt; nút `Đặt lại` `Huỷ` | CẦN CHỐT: có xoá gõ tắt và danh sách loại trừ không? Nếu không, thêm `Gõ tắt và danh sách loại trừ được giữ.` |
| Đặt lại tất cả — xong | `Đã đặt lại tất cả cài đặt` | 25/72 | toast + `Hoàn tác` | 10 s |
| Tệp cài đặt hỏng, đã dùng mặc định | `Tệp cài đặt bị hỏng nên VietKB dùng cài đặt mặc định. Bản cũ vẫn được giữ lại.` | 78/110 | thông báo Windows lúc mở + dải vàng trên tab Gõ | nút `Mở thư mục`; UI-45 |
| Bộ gõ dừng và đã tự chạy lại | `Bộ gõ vừa dừng đột ngột và đã tự chạy lại. Bạn cứ gõ tiếp.` | 58/110 | thông báo Windows | CẦN CHỐT: spec C.1 chỉ có nút chạy lại tay, chưa có tự chạy lại |
| Bộ gõ dừng, chưa chạy lại được | `Bộ gõ đang không chạy nên chưa gõ được tiếng Việt.` | 50/110 | dải đỏ trên tab Gõ | nút `Chạy lại` (thay `Lỗi hook — Khởi động lại`) |
| Có bản mới | `Đã có VietKB 0.2.0, bạn đang dùng 0.1.0` | 39/72 | thông báo Windows + dòng mô tả Tự kiểm tra cập nhật | nút `Xem bản mới` (mở trang tải) |
| Đang là bản mới nhất | `Bạn đang dùng bản mới nhất` | 26/72 | thay mô tả dòng cập nhật 3 s | |
| Kiểm tra cập nhật lỗi | `Chưa kiểm tra được, có thể máy đang mất mạng` | 44/72 | mô tả dòng cập nhật | nút `Thử lại` |
| Bật quyền Admin, chờ mở lại | `Cần mở lại VietKB, Windows sẽ hỏi xác nhận · Mở lại ngay` | 56/64 | mô tả dòng | (§3.4) |
| Ứng dụng đang dùng chạy quyền Admin | `Ứng dụng này chạy quyền Admin nên chưa gõ được tiếng Việt · Bật quyền Admin` | 75/110 | CẦN CHỐT vị trí: tab Tổng quan (spec C.1) đã bỏ ở v3 | |
| Có bộ gõ khác đang chạy | `UniKey đang chạy cùng lúc nên dễ gõ sai dấu. Hãy tắt UniKey.` | 60/110 | dải vàng trên tab Gõ | spec C.10; tên lấy từ ứng dụng phát hiện |
| Khởi động cùng Windows lỗi | `Chưa bật được Khởi động cùng Windows: Windows từ chối thay đổi.` | 63/110 | dưới dòng | spec C.7 |
| Đã nhập cài đặt cũ | `Đã chuyển cài đặt từ bản cũ sang` | 32/72 | toast lần mở đầu | spec C.10 |
| Đã lưu | `Đã lưu` | 6/16 | cạnh dòng 1,2 s | |

---

## §5 Menu ở biểu tượng VietKB và thông báo

Menu chuột phải là chỗ dùng nhiều nhất: mỗi mục là một động từ hoặc một tên quen, đọc trong nửa giây.
Phím tắt in mờ bên phải, không tính vào 24 ký tự.

| vị trí | chuỗi | độ dài | ghi chú |
|---|---|---|---|
| Mục 1 (có dấu ✓) | `Bật tiếng Việt` | 14/24 | phải: `Ctrl+Shift` |
| Mục 2 (menu con) | `Kiểu gõ` | 7/24 | Telex · VNI · VIQR |
| Mục 3 (menu con) | `Bảng mã` | 7/24 | tên đầy đủ như §3.1 |
| Mục 4 (✓) | `Bật gõ tắt` | 10/24 | thay `Gõ tắt` — cùng mẫu với `Bật tiếng Việt` |
| Mục 5 (✓) | `Kiểm tra chính tả` | 17/24 | |
| Mục 6 (✓ khi đã loại trừ) | `Loại trừ Chrome` | 15/24 | `Loại trừ ` + tên ứng dụng phía trước, cắt ở 15 ký tự |
| Mục 6, tên dài | `Loại trừ Visual Studio…` | 23/24 | ví dụ cắt: tên + `…` ≤ 15 ký tự, bỏ dấu cách cuối trước `…` |
| Mục 6, không có ứng dụng | `Loại trừ ứng dụng này` | 21/24 | mờ khi cửa sổ phía trước là màn hình nền hoặc VietKB |
| Mục 7 | `Bảng gõ tắt…` | 12/24 | tên EVKey |
| Mục 8 | `Mở cài đặt…` | 11/24 | |
| Mục 9 | `Chuyển mã…` | 10/24 | Sprint 2 |
| Mục cuối | `Thoát VietKB` | 12/24 | thay `Thoát` — rõ là tắt hẳn, khác với `Bật tiếng Việt` |
| Tooltip, đang bật | `VietKB · Tiếng Việt · Telex · Unicode` | 37/64 | spec C.9 (đổi `—` thành `·`) |
| Tooltip, đang tắt | `VietKB · Đã tắt tiếng Việt` | 26/64 | |
| Thông báo khi bật | `Đã bật tiếng Việt` | 17/72 | `notify_on_toggle` |
| Thông báo khi tắt | `Đã tắt tiếng Việt` | 17/72 | |
| Chữ nổi cạnh con trỏ | `V` / `E` | — | giữ (spec B.2 overlay) |
| Đóng cửa sổ lần đầu | `VietKB vẫn chạy. Bấm biểu tượng cạnh đồng hồ để mở lại.` | 55/72 | thông báo một lần |

Từ chỉ trạng thái dùng thống nhất: **bật** → `Tiếng Việt`; **tắt** → `Đã tắt tiếng Việt`.
Không dùng `English`, `Tiếng Anh`, `chế độ E` ở bất cứ đâu.

---

## §6 Lần mở đầu tiên

Cửa sổ mở ở tab **Gõ**. Không trình hướng dẫn, không hộp thoại. Một khung gợi ý nhỏ (chỗ đặt: do thiết kế
quyết; spec C.10 gọi là "inline card") với tối đa 3 dòng và một nút; bấm nút thì không hiện lại.

| vị trí | chuỗi | độ dài | ghi chú |
|---|---|---|---|
| Dòng mở đầu | `VietKB đã sẵn sàng. Ba điều nên biết:` | 37/70 | |
| Gợi ý 1 | `Nhấn rồi nhả Ctrl + Shift để bật hoặc tắt tiếng Việt` | 52/70 | phím lấy theo cài đặt |
| Gợi ý 2 | `Thử ngay ở ô Thử gõ: tieengs Vieetj → tiếng Việt` | 48/70 | đổi theo kiểu gõ: VNI `tie61ng Vie65t` |
| Gợi ý 3 | `Đóng cửa sổ thì VietKB vẫn chạy ở biểu tượng cạnh đồng hồ` | 57/70 | |
| Nút | `Đã hiểu` | 7/16 | thay `Xong` của spec — đây là gợi ý, không phải bước |

Cùng lúc, nếu có: dải vàng `UniKey đang chạy cùng lúc…` (§4) và toast `Đã chuyển cài đặt từ bản cũ sang`.
Từ lần mở thứ hai: chỉ có biểu tượng, không mở cửa sổ (spec C.10).

---

## §7 Soát chữ hiện có (4 tệp HTML)

`giữ` = dùng nguyên. `sửa` = thay bằng chuỗi ở cột 4 (đã có trong §3/§4 kèm độ dài).

### go.html

| # | Chuỗi hiện tại | Kết luận | Thay bằng | Lý do |
|---|---|---|---|---|
| 1 | `VietKB — Gõ (sáng)` (tiêu đề) | sửa | `Cài đặt VietKB` | `(sáng)` là nhãn bản mẫu; tiêu đề thật phải nói đây là cửa sổ gì |
| 2 | `Gõ` | giữ | | |
| 3 | `Gõ tắt` + `148` | giữ | | |
| 4 | `Phím tắt` | giữ | | |
| 5 | `Hệ thống` | giữ | | |
| 6 | `Tiếng Việt` (công tắc) | giữ | | |
| 7 | `Ctrl` `+` `Shift` | giữ | | |
| 8 | `Kiểu gõ` | giữ | | |
| 9 | `Telex` | giữ | | |
| 10 | `Bảng mã` | giữ | | |
| 11 | `Unicode dựng sẵn` | giữ | | |
| 12 | `Kiểu đặt dấu` | giữ | | |
| 13 | `Vị trí dấu thanh trong vần oa, oe, uy` | sửa | `Hai kiểu chỉ khác nhau ở vần oa, oe, uy` | "vị trí dấu thanh" là từ ngữ pháp; nói điều quan trọng: chỉ khác 3 vần |
| 14 | `Kiểu mới · hòa, thúy` | giữ | | |
| 15 | `Kiểu cũ · hoà, thuý` | giữ | | |
| 16 | `Bỏ dấu ở cuối từ` | sửa | `Gõ dấu ở cuối từ` | `bỏ dấu` còn nghĩa "xoá dấu" |
| 17 | `hoangf → hoàng: gõ dấu sau cả từ` | sửa | `hoangf → hoàng: gõ dấu sau cả từ cũng được` | câu cũ đọc như *bắt buộc*; tuỳ chọn này là *cho phép* |
| 18 | `Tự sửa ưo thành ươ` | giữ | | |
| 19 | `nguwoif → người, chỉ cần một w` | giữ | | đúng spec A1, ví dụ rõ |
| 20 | `Kiểm tra chính tả` | giữ | | |
| 21 | `Không đặt dấu vào từ tiếng Anh như “search”, “excel”` | sửa | `Chỉ thêm dấu khi ra từ tiếng Việt: search vẫn là search` | câu phủ định; `excel` có thể vẫn ra `ẽcel` khi tắt khôi phục (spec A9 chỉ chặn phím *sau* chỗ sai) — ví dụ không được hứa sai |
| 22 | `Tự động khôi phục từ gõ sai` | giữ | | |
| 23 | `Cần bật Kiểm tra chính tả · Bật` (trạng thái mờ) | sửa | `gôgle tự trở về google khi bạn gõ dấu cách` (dòng bật được) | spec A10 "regardless of spell_check" và C.2 "Disabled state: none" — phụ thuộc này không có thật; dòng thiếu luôn câu giải thích |
| 24 | `Bật` (liên kết sửa nhanh) | giữ | | mẫu đúng; dùng cho phụ thuộc thật (vd. Viết hoa theo từ tắt) |
| 25 | `Bảo vệ đường dẫn và mã` | sửa | `Giữ nguyên email, địa chỉ web` | `mã` đụng `bảng mã`; "bảo vệ" không nói điều gì xảy ra |
| 26 | `Không thêm dấu khi gõ URL, email, mã lệnh` | sửa | `ten@congty.vn, abc.com/xyz gõ sao ra vậy, không thêm dấu` | `URL`, `mã lệnh` là từ kỹ thuật; ví dụ nói thay |
| 27 | `Viết hoa sau dấu câu` | sửa | `Viết hoa đầu câu` | cũng viết hoa sau xuống dòng — "sau dấu câu" thiếu |
| 28 | `Tự viết hoa chữ đầu sau . ! ? và khi xuống dòng` | sửa | `chào. tôi → chào. Tôi, cả sau ! ? và khi xuống dòng` | ví dụ thay luật |
| 29 | `Gõ tắt` (dòng) | giữ | | |
| 30 | `Gõ nth rồi Space thành Nguyễn Thu Hà` | sửa | `Gõ nth rồi dấu cách → Nguyễn Thu Hà` | câu cũ đọc thành "Space thành Nguyễn Thu Hà"; `Space` là chữ Anh giữa câu |
| 31 | `148 mục ›` | giữ | | |
| 32 | `Nâng cao` | giữ | | |
| 33 | `8 tuỳ chọn` | giữ | | |
| 34 | `f j w z làm phụ âm, gõ tắt khi tắt tiếng Việt, độ trễ gửi phím…` | sửa | `f j w z làm phụ âm, sửa lỗi dấu gợi ý, Metro, Clipboard` | có `…` cuối (cắt chữ); `độ trễ gửi phím` là từ kỹ thuật; nêu các tên EVKey người dùng hay tìm |
| 35 | `Thử gõ` | giữ | | |
| 36 | `Nguyễn Thu Hà ở Đường Nguyễn Văn Linh, Quận 7` (mẫu) | giữ | | dữ liệu mẫu, đúng Telex |
| 37 | `Telex · Unicode` | giữ | | |

### macro.html

| # | Chuỗi hiện tại | Kết luận | Thay bằng | Lý do |
|---|---|---|---|---|
| 38 | `VietKB — Gõ tắt (sáng)` | sửa | `Cài đặt VietKB` | như #1 |
| 39 | `Tiếng Việt · Telex · Unicode` (chip) | giữ | | |
| 40 | `Tìm từ tắt hoặc nội dung` | giữ | | |
| 41 | `Ctrl` `F` | giữ | | |
| 42 | `Nhập…` | sửa | `Nhập từ tệp…` | `Nhập` cạnh `Thêm` đọc thành "gõ vào" |
| 43 | `Xuất…` | sửa | `Xuất ra tệp…` | đi cặp với #42 |
| 44 | `Thêm` | giữ | | |
| 45 | `Đã chọn 1 trong 148` | giữ | | |
| 46 | `Sửa` | giữ | | |
| 47 | `Xuất` (thanh chọn) | sửa | `Xuất…` | mở hộp chọn nơi lưu → cần `…` |
| 48 | `Xoá` | giữ | | |
| 49 | `Bỏ chọn` + `Esc` | giữ | | |
| 50 | `+1 dòng` | giữ | | |
| 51 | Dữ liệu mẫu 8 mục (`ctdd`, `dc`, `dchi`, `hcm`, `kg`, `mail`, `mst`, `nth`) | giữ | | là chữ của người dùng; `Kính gửi Quý khách hàng,` là nội dung của họ, không phải giọng sản phẩm |
| 52 | Nút trên hàng: bút, `…` (không tooltip) | sửa | tooltip `Sửa` · `Tuỳ chọn khác` | nút chỉ có hình phải có chữ khi rê chuột |
| 53 | `Hiện 8 trong 148 mục · gõ để lọc` | sửa | `Đang hiện 8 trong 148 mục · gõ vào ô tìm để lọc` | "gõ để lọc" không nói gõ vào đâu — người dùng có thể gõ vào bảng |
| 54 | `Mở bảng đầy đủ ›` | sửa | `Mở bảng gõ tắt` | tên EVKey; trùng mục menu `Bảng gõ tắt…` |
| 55 | Tiêu đề cột (thiết kế chưa có, đang bị thanh chọn che) | sửa | `Từ tắt` · `Nội dung` | cần cho trạng thái không chọn |

### hotkey.html

| # | Chuỗi hiện tại | Kết luận | Thay bằng | Lý do |
|---|---|---|---|---|
| 56 | `VietKB — Phím tắt (sáng)` | sửa | `Cài đặt VietKB` | như #1 |
| 57 | `Nhấn vào ô rồi bấm tổ hợp mới. Esc huỷ, Backspace bỏ trống.` | sửa | `Bấm vào ô, nhấn tổ hợp mới. Esc để huỷ, Backspace để xoá.` | đảo `nhấn`/`bấm` (§1); `bỏ trống` → `xoá` |
| 58 | `Chuyển chế độ` | giữ | | |
| 59 | `Bật/tắt tiếng Việt` | giữ | | |
| 60 | `Ctrl + Shift + T trong trình duyệt vẫn mở lại tab` | giữ | | ví dụ cụ thể cho "nhả mới chuyển"; cần bản thứ hai khi tổ hợp có phím thường (§3.3) |
| 61 | `Nhả để chuyển` | giữ | | |
| 62 | `Đổi bảng mã` | giữ | | |
| 63 | `Unicode → TCVN3 → VNI Windows` | giữ | | nội dung vòng đổi CẦN CHỐT |
| 64 | `Esc` `huỷ` (đang nghe phím) | giữ | | |
| 65 | `Đổi kiểu gõ` | giữ | | |
| 66 | `Telex → VNI → VIQR` | giữ | | |
| 67 | `Mẫu cho Bật/tắt:` | sửa | `Chọn nhanh:` | "mẫu" là từ thiết kế; dòng đã nằm ngay dưới nhóm bật/tắt |
| 68 | `mặc định` | giữ | | |
| 69 | `như UniKey` | giữ | | |
| 70 | `Right Ctrl` | sửa | `Ctrl phải` | chữ Anh; người dùng nói "Ctrl bên phải" |
| 71 | `Công cụ` | giữ | | |
| 72 | `Khôi phục từ gõ sai` | giữ | | |
| 73 | `Đưa từ vừa gõ về đúng các phím đã bấm` | sửa | `Đưa từ vừa gõ về đúng phím đã nhấn: gôgle → google` | thêm ví dụ; `bấm` → `nhấn` cho phím |
| 74 | `Trùng “Làm lại” của Word. Không có từ để khôi phục thì phím vẫn đến ứng dụng.` | sửa | `Cũng là phím Làm lại ở nhiều ứng dụng; không có từ để khôi phục thì vẫn Làm lại` | Word dùng Ctrl + Y để Làm lại, không phải Ctrl + Shift + Z (spec AC-46 nói Notepad++, VS Code) — sai sự thật; "phím vẫn đến ứng dụng" là câu dịch |
| 75 | `Trùng` (thẻ vàng) | sửa | `Dễ trùng` | đây là cảnh báo cho phép lưu; `Trùng` để dành cho xung đột chặn giữa hai dòng |
| 76 | `Mở cài đặt` | giữ | | |
| 77 | `Đã đổi từ Ctrl + Alt + S` | sửa | `Mặc định là Ctrl + Alt + S` | nói thẳng nút đặt lại sẽ đưa về đâu |
| 78 | `Về mặc định` (tooltip) | sửa | `Đặt lại mặc định` | thống nhất `đặt lại` (§1) |
| 79 | `Chuyển mã clipboard` | sửa | `Chuyển mã chữ đã chép` | `clipboard` viết thường giữa câu, là từ kỹ thuật |
| 80 | `Đổi văn bản đã chép sang bảng mã đang dùng` | sửa | `Đổi chữ đã chép sang bảng mã đang dùng, rồi dán như thường` | nói bước tiếp theo; nguồn đổi CẦN CHỐT |
| 81 | `Chưa đặt` | giữ | | |
| 82 | `Đặt phím` | giữ | | |
| 83 | `Khôi phục tất cả` | sửa | `Đặt lại tất cả` | `khôi phục` đã là "khôi phục từ gõ sai" ngay trên màn hình này |
| 84 | `Không gán được:` | sửa | `Windows đã dùng:` | `gán` là từ kỹ thuật; nói lý do |
| 85 | `Win` `L` · `Alt` `Tab` · `Ctrl` `Alt` `Del` | giữ | | |

### system.html

| # | Chuỗi hiện tại | Kết luận | Thay bằng | Lý do |
|---|---|---|---|---|
| 86 | `VietKB — Hệ thống (sáng)` | sửa | `Cài đặt VietKB` | như #1 |
| 87 | `Khởi động` | giữ | | |
| 88 | `Khởi động cùng Windows` | giữ | | nhãn EVKey |
| 89 | `Chạy ở khay hệ thống, khoảng 14 MB bộ nhớ` | sửa | `Bật máy là gõ được ngay, biểu tượng nằm cạnh đồng hồ` | `khay hệ thống` lạ với người thường; "14 MB" không giúp quyết định và có thể sai (ngân sách P3 là 20 MB) |
| 90 | `Chạy với quyền Admin` | giữ | | nhãn EVKey |
| 91 | `Có hiệu lực sau khi khởi động lại · Khởi động lại ngay` | sửa | `Cần mở lại VietKB, Windows sẽ hỏi xác nhận · Mở lại ngay` | "khởi động lại" bị hiểu là khởi động lại máy; báo trước hộp hỏi của Windows |
| 92 | (mô tả khi tắt — chưa có) | sửa | `Cần khi gõ vào ứng dụng cũng chạy quyền Admin` | người dùng phải biết khi nào cần bật |
| 93 | `Giao diện` (mục) | giữ | | |
| 94 | `Âm thanh khi bật/tắt` | giữ | | |
| 95 | `Tiếng tách nhỏ mỗi lần chuyển Việt/Anh` | sửa | `Kêu “tách” nhỏ mỗi lần bật hoặc tắt tiếng Việt` | "Việt/Anh" mâu thuẫn với §1 (tắt ≠ tiếng Anh) |
| 96 | `Ngôn ngữ` | sửa | `Ngôn ngữ hiển thị` | trong bộ gõ, "ngôn ngữ" dễ hiểu là ngôn ngữ gõ |
| 97 | `Áp dụng ngay, không cần mở lại` | sửa | `Chữ trên cửa sổ và menu của VietKB, không đổi cách gõ` | mọi cài đặt đều áp dụng ngay — câu cũ không giúp phân biệt; câu mới trả lời nỗi lo "đổi sang English có hỏng gõ không" |
| 98 | `Tiếng Việt` · `English` | giữ | | |
| 99 | `Giao diện` (dòng) | sửa | `Màu nền` | trùng tên mục chứa nó |
| 100 | `Sáng, tối hoặc theo Windows` | sửa | `Theo Windows: tự đổi khi Windows chuyển sáng hoặc tối` | câu cũ lặp lại 3 nút ngay bên cạnh |
| 101 | `Theo hệ thống` | sửa | `Theo Windows` | cụ thể hơn, khớp mô tả |
| 102 | `Sáng` · `Tối` | giữ | | |
| 103 | `Cập nhật` | giữ | | |
| 104 | `Tự kiểm tra cập nhật` | giữ | | |
| 105 | `Phiên bản 0.1.0 · chỉ hỏi phiên bản mới, không gửi dữ liệu` | sửa | `Đang dùng bản 0.1.0 · chỉ xem có bản mới, không gửi gì khác` | "dữ liệu" mơ hồ; lặp "phiên bản" |
| 106 | `Kiểm tra ngay` | giữ | | |
| 107 | `Tệp cấu hình` | sửa | `Tệp cài đặt` | thống nhất `cài đặt` (§1) |
| 108 | `%APPDATA%\VietKB\config.toml` | giữ | | |
| 109 | `Mở thư mục` | giữ | | |
| 110 | `Khôi phục…` | sửa | `Đặt lại tất cả…` | `khôi phục` gợi "lấy lại bản cũ" — ngược nghĩa với xoá cài đặt; không nói phạm vi |
| 111 | `Loại trừ ứng dụng` | giữ | | nhãn EVKey |
| 112 | `6 ứng dụng` | giữ | | |
| 113 | `Quản lý ›` | giữ | | |
| 114 | `Windows Terminal` · `Visual Studio Code` · `AutoCAD 2024` | giữ | | tên ứng dụng |
| 115 | `Tắt tiếng Việt` (chế độ) | giữ | | |
| 116 | `Bảng mã TCVN3` (chế độ) | giữ | | |

**Tổng:** 116 mục soát · 46 `sửa` (trong đó 4 là tiêu đề bản mẫu, 3 là chuỗi còn thiếu: #52, #55, #92) · 70 `giữ`.
Các chuỗi khác dòng 3 trong hàng (cảnh báo, lỗi dưới ô) cũng giới hạn thực ~ 80 ký tự vì chạy trong một dòng 12 px;
mọi chuỗi lỗi ở §4 đặt dưới ô đều ≤ 80, trừ các thông báo Windows/toast/dải trên cùng (được tới 110).

---

## §8 CẦN CHỐT (gom một chỗ)

| # | Chỗ | Cần gì |
|---|---|---|
| 1 | Nâng cao: `Tắt khi đổi bàn phím khác` | Có trong DESIGN-SPEC, không có trong spec B.2. Có làm không? Nếu có: tên khoá TOML và "bàn phím không phải US" gồm những gì. |
| 2 | Nâng cao: `Độ trễ giữa các phím` | Đơn vị hiển thị (µs? ms?) và mặc định (spec 0, thiết kế 8). |
| 3 | Phím tắt: `Đổi bảng mã` | Vòng đổi gồm bảng mã nào, thứ tự nào. |
| 4 | Phím tắt: `Chuyển mã chữ đã chép` | Đổi *từ* bảng mã nào (tự nhận biết? hỏi người dùng?). |
| 5 | Bảng mã `UTF-8 Literal` | Ý nghĩa (spec A11 tự ghi `UNVERIFIED`) — nếu là UTF-8 thường thì bỏ khỏi danh sách (trùng Unicode). |
| 6 | Bảng mã `BK HCM1`, `VPS` | Ngoài Sprint 1: ẩn hay hiện mờ. |
| 7 | Trùng phím quen của ứng dụng (cảnh báo vàng) | Khi đặt Ctrl + C làm phím tắt, ứng dụng còn nhận Ctrl + C không? Quyết định câu hậu quả. |
| 8 | Nhập tệp gõ tắt | Dạng tệp (EVKey `.txt`? CSV? tab?) — quyết định câu lỗi và dòng hướng dẫn. |
| 9 | Nội dung gõ tắt trống | Hợp lệ hay lỗi (spec C.4 không nói). |
| 10 | Đặt lại tất cả | Có xoá gõ tắt và danh sách loại trừ không. |
| 11 | Bộ gõ tự chạy lại | Spec chỉ có nút chạy lại tay (C.1); tự chạy lại có làm không. |
| 12 | Ghi chú "ứng dụng chạy quyền Admin" | Tab Tổng quan (spec C.1) đã bỏ ở v3 — câu này hiện ở đâu (chip trạng thái? dải trên tab Gõ? thông báo?). |
| 13 | Menu `…` trên hàng gõ tắt | Gồm những lệnh nào. |
| 14 | Trạng thái tắt trong spec C.0 (`English`) | Đề xuất đổi thành `Đã tắt tiếng Việt` (§1) — cần Lead đồng ý vì spec ghi khác. |
