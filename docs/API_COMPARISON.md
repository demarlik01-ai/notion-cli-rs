# Notion API 분석 및 비교

> 문서 작성일: 2026-02-02  
> 참조: https://developers.notion.com/

## API 버전

| 항목 | 현재 구현 | 최신 버전 |
|------|----------|----------|
| Notion-Version | `2022-06-28` | `2025-09-03` |

### ⚠️ 버전 업그레이드 필요

현재 구현이 구버전 API를 사용 중. 최신 버전에서 변경된 사항:
- Database와 Data Source 개념 분리
- `children` → `content` 파라미터 변경 (페이지 생성 시)
- 새로운 블록 타입 추가

**권장**: `.env`에서 `NOTION_API_VERSION` 설정 가능하도록 이미 구현됨. 기본값을 `2025-09-03`으로 업데이트 고려.

---

## 구현 현황

### ✅ 구현된 기능

| 기능 | 엔드포인트 | 상태 |
|------|-----------|------|
| 검색 | `POST /search` | ✅ 페이지네이션 지원 |
| 페이지 조회 | `GET /pages/{id}` | ✅ |
| 블록 조회 | `GET /blocks/{id}/children` | ✅ 페이지네이션 지원 |
| 페이지 생성 | `POST /pages` | ✅ 기본 기능 |
| 블록 추가 | `PATCH /blocks/{id}/children` | ✅ paragraph만 |

### ❌ 미구현 기능

| 기능 | 엔드포인트 | 우선순위 |
|------|-----------|---------|
| 페이지 수정 | `PATCH /pages/{id}` | 🔴 높음 |
| 페이지 삭제/아카이브 | `PATCH /pages/{id}` (archived=true) | 🔴 높음 |
| 블록 수정 | `PATCH /blocks/{id}` | 🟡 중간 |
| 블록 삭제 | `DELETE /blocks/{id}` | 🟡 중간 |
| 데이터베이스 조회 | `GET /databases/{id}` | 🟡 중간 |
| 데이터베이스 쿼리 | `POST /databases/{id}/query` | 🔴 높음 |
| 데이터베이스 생성 | `POST /databases` | 🟢 낮음 |
| 댓글 조회 | `GET /comments` | 🟢 낮음 |
| 댓글 생성 | `POST /comments` | 🟢 낮음 |
| 사용자 목록 | `GET /users` | 🟢 낮음 |
| 사용자 조회 | `GET /users/{id}` | 🟢 낮음 |

---

## Best Practices 체크리스트

### ✅ 잘 구현된 것

- [x] **UUID 정규화**: 하이픈 있는/없는 ID 모두 처리
- [x] **페이지네이션**: search, get_blocks에서 자동 처리
- [x] **타임아웃 설정**: CLI 옵션으로 조정 가능
- [x] **환경 변수**: .env 파일 지원
- [x] **에러 컨텍스트**: anyhow로 에러 체이닝

### ⚠️ 개선 필요

- [ ] **Rate Limiting**: 429 응답 시 `Retry-After` 헤더 확인 및 재시도 로직 없음
- [ ] **에러 코드 파싱**: API 에러 응답의 `code`, `message` 필드 파싱 없음
- [ ] **Rich Text 지원**: plain text만 생성, 서식(bold, italic 등) 미지원
- [ ] **다양한 블록 타입**: paragraph만 생성 가능

---

## Rate Limits

| 제한 | 값 |
|-----|-----|
| 요청 속도 | 평균 3 req/sec |
| 페이로드 크기 | 최대 500KB |
| 블록 개수 | 요청당 최대 1000개 |
| Rich text 길이 | 최대 2000자 |
| URL 길이 | 최대 2000자 |
| 배열 요소 | 최대 100개 |

### 권장 구현

```rust
// 429 응답 처리 예시
if response.status() == 429 {
    if let Some(retry_after) = response.headers().get("Retry-After") {
        let secs: u64 = retry_after.to_str()?.parse()?;
        std::thread::sleep(Duration::from_secs(secs));
        // 재시도
    }
}
```

---

## 지원되는 블록 타입 (전체 목록)

현재 CLI가 **읽을 수 있는** 블록:
- ✅ paragraph
- ✅ heading_1, heading_2, heading_3
- ✅ bulleted_list_item, numbered_list_item
- ✅ code
- ✅ divider

**추가 지원 가능한** 블록:
- 📝 quote
- ✔️ to_do (체크박스)
- 🔗 bookmark
- 📞 callout
- 📊 table, table_row
- 🖼️ image, video, file, pdf
- ➗ equation
- 🔄 synced_block
- 📑 toggle
- 🗂️ column_list, column

---

## 추천 개선 사항

### 1단계 (핵심 기능)

```bash
# 페이지 수정
notion update <page_id> --title "새 제목"
notion update <page_id> --icon "🚀"

# 페이지 삭제/복원
notion delete <page_id>
notion restore <page_id>

# 데이터베이스 쿼리
notion query <database_id> --filter "Status=Done"
```

### 2단계 (확장 기능)

```bash
# 다양한 블록 타입 추가
notion append <page_id> --type heading_1 "제목"
notion append <page_id> --type todo "할 일"
notion append <page_id> --type code --lang rust "fn main() {}"

# 블록 수정/삭제
notion block update <block_id> "새 내용"
notion block delete <block_id>
```

### 3단계 (고급 기능)

```bash
# 댓글
notion comment <page_id> "댓글 내용"
notion comments <page_id>

# 사용자
notion users
notion whoami
```

---

## 참고 링크

- [API Reference](https://developers.notion.com/reference/intro)
- [Versioning](https://developers.notion.com/reference/versioning)
- [Request Limits](https://developers.notion.com/reference/request-limits)
- [Status Codes](https://developers.notion.com/reference/status-codes)
- [Block Types](https://developers.notion.com/reference/block)
- [Upgrade Guide 2025-09-03](https://developers.notion.com/guides/get-started/upgrade-guide-2025-09-03)
