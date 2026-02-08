# 아키텍처

## 개요

notion-cli는 Notion REST API를 터미널에서 사용할 수 있게 해주는 Rust CLI 애플리케이션입니다. 6개 소스 파일(총 ~1,700줄)로 구성된 모듈 구조를 따릅니다.

## 프로젝트 구조

```
notion-cli-rs/
├── src/
│   ├── main.rs        # 진입점, 명령어 라우팅, init/config 핸들러
│   ├── cli.rs         # CLI 인자 정의 (clap derive)
│   ├── client.rs      # NotionClient - HTTP 클라이언트 & API 메서드
│   ├── commands.rs    # 명령어 핸들러 함수
│   ├── render.rs      # 터미널 출력 포맷팅
│   └── utils.rs       # 설정 관리, 헬퍼, 상수
├── docs/
│   ├── ARCHITECTURE.md
│   ├── ARCHITECTURE-ko.md
│   └── API_COMPARISON.md
├── Cargo.toml
└── README.md
```

## 모듈 다이어그램

```
                    ┌──────────────┐
                    │   main.rs    │
                    │  - CLI 파싱  │
                    │  - 라우팅    │
                    │  - init/cfg  │
                    └──────┬───────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
       ┌───────────┐ ┌──────────┐ ┌──────────┐
       │  cli.rs   │ │commands.rs│ │ utils.rs │
       │  (clap)   │ │(핸들러)   │ │  (설정)  │
       └───────────┘ └────┬─────┘ └──────────┘
                          │
                          ▼
                   ┌─────────────┐
                   │ client.rs   │
                   │(NotionClient)│
                   └──────┬──────┘
                          │
                   ┌──────┴──────┐
                   ▼             ▼
            ┌───────────┐ ┌───────────┐
            │ render.rs │ │ Notion API│
            │  (출력)   │ │  (REST)   │
            └───────────┘ └───────────┘
```

## 모듈 설명

### `cli.rs` — CLI 정의

clap의 derive API를 사용한 CLI 구조 정의.

- `Cli` 구조체: 글로벌 옵션 (`--api-key`, `--timeout`)
- `Commands` 열거형: 18개 서브커맨드 (search, read, create, append, update, delete, query, move, init, config 등)

### `main.rs` — 진입점 & 라우팅

1. CLI 인자 파싱
2. `init`, `config` 명령어 처리 (API 키 불필요)
3. 우선순위 체인으로 API 키 확인
4. `NotionClient` 초기화
5. 적절한 명령어 핸들러로 라우팅

`handle_init()`과 `handle_config_with_cli_key()`도 포함.

### `client.rs` — Notion API 클라이언트

`NotionClient`는 reqwest의 blocking HTTP 클라이언트를 래핑.

**주요 기능:**
- Bearer 토큰 인증
- Notion-Version 헤더 (`2025-09-03`)
- 검색/블록 조회 시 자동 페이지네이션
- Rate limit(HTTP 429) 시 지수 백오프 자동 재시도
- 리치 텍스트 빌더 헬퍼 (`plain`, `link`, `code_inline`, `bold`)

**API 메서드 (16개):**
| 메서드 | HTTP | 엔드포인트 |
|--------|------|-----------|
| `search` | POST | `/search` |
| `get_page` | GET | `/pages/{id}` |
| `get_blocks` | GET | `/blocks/{id}/children` |
| `create_page` | POST | `/pages` |
| `append_blocks` | PATCH | `/blocks/{id}/children` |
| `update_page` | PATCH | `/pages/{id}` |
| `delete_page` | PATCH | `/pages/{id}` (아카이브) |
| `append_code_block` | PATCH | `/blocks/{id}/children` |
| `append_bookmark` | PATCH | `/blocks/{id}/children` |
| `delete_block` | DELETE | `/blocks/{id}` |
| `append_heading` | PATCH | `/blocks/{id}/children` |
| `append_rich_text` | PATCH | `/blocks/{id}/children` |
| `append_divider` | PATCH | `/blocks/{id}/children` |
| `append_bulleted_list` | PATCH | `/blocks/{id}/children` |
| `query_database` | POST | `/databases/{id}/query` |
| `move_page` | POST+PATCH | `/pages` + `/pages/{id}` |

### `commands.rs` — 명령어 핸들러

각 핸들러 함수의 흐름:
1. 입력 검증 및 정규화 (페이지 ID 등)
2. `NotionClient` 메서드 호출
3. `render.rs`를 통한 출력 포맷팅

CLI 서브커맨드에 대응하는 16개 핸들러 함수.

### `render.rs` — 출력 포맷팅

`colored` 크레이트를 사용한 터미널 렌더링:
- `extract_title()` — Notion 페이지/데이터베이스 객체에서 제목 추출
- `extract_rich_text()` — 블록 rich_text 배열에서 텍스트 추출
- `extract_property_value()` — 데이터베이스 쿼리용 프로퍼티 값 추출
- `print_block()` — 블록 타입별 포맷팅 및 출력

**지원 블록 타입:** paragraph, heading (1-3), bulleted/numbered list, code, divider, bookmark, to-do

### `utils.rs` — 설정 & 헬퍼

**설정 관리:**
- `Config` 구조체: `api_key`, `timeout` (TOML로 직렬화)
- 설정 경로: `~/.config/notion-cli/config.toml`
- `load_config()` / `save_config()` — TOML 읽기/쓰기

**API 키 확인 우선순위:**
1. `--api-key` CLI 옵션
2. `NOTION_API_KEY` 환경변수
3. `~/.config/notion-cli/config.toml`
4. `.env` 파일 (하위호환)

**기타 유틸리티:**
- `normalize_page_id()` — 다양한 ID 형식을 UUID로 변환
- `get_api_version()` — API 버전 문자열

## 의존성

| 크레이트 | 용도 |
|----------|------|
| `clap` | CLI 인자 파싱 (derive) |
| `reqwest` | HTTP 클라이언트 (blocking, rustls-tls) |
| `serde` / `serde_json` | JSON 직렬화 |
| `toml` | 설정 파일 파싱 |
| `dirs` | XDG 설정 디렉토리 확인 |
| `dotenvy` | .env 파일 로딩 (레거시 폴백) |
| `anyhow` | 컨텍스트 포함 에러 처리 |
| `colored` | 터미널 색상 출력 |

## 에러 처리

- 모든 함수가 `anyhow::Result<T>` 반환
- 모든 API 호출에 `.context()` 적용으로 명확한 에러 메시지
- Rate limit (HTTP 429): 지수 백오프 자동 재시도 (최대 3회)
- `main()`에서 통합 에러 핸들링 → 빨간 `✗` 출력 → 종료 코드 1

## 설계 결정

- **Blocking HTTP**: 순차적 CLI 도구에서 async는 불필요
- **모듈 분리**: ~800줄 시점에서 단일 파일을 분리하여 유지보수성 향상
- **글로벌 설정**: 이식성을 위해 `.env` 대신 XDG 표준 `~/.config/` 사용
- **자동 페이지네이션**: 사용자가 커서를 직접 다룰 필요 없음
- **데이터베이스 아이템 생성 미지원**: 페이지만 — API 표면을 단순하게 유지
