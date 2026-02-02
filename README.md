# notion-cli-rs

Rust로 작성된 Notion CLI 도구. 터미널에서 Notion 페이지와 데이터베이스를 관리합니다.

## 기능

| 명령어 | 설명 |
|--------|------|
| `search` | 페이지/데이터베이스 검색 |
| `read` | 페이지 내용 읽기 |
| `create` | 새 페이지 생성 |
| `append` | 페이지에 내용 추가 |
| `update` | 페이지 제목/아이콘 수정 |
| `delete` | 페이지 삭제 (휴지통으로 이동) |
| `query` | 데이터베이스 쿼리 |

## 설치

### 사전 요구사항
- Rust 1.70+
- Notion Integration Token ([발급하기](https://notion.so/my-integrations))

### 빌드

```bash
# 개발 빌드
cargo build

# 릴리스 빌드 (권장)
cargo build --release
```

### 전역 설치

```bash
cargo install --path .
```

## 설정

```bash
cp .env.example .env
```

`.env` 파일 수정:
```
NOTION_API_KEY=ntn_xxxxx
```

## 사용법

### 검색

```bash
# 기본 검색
notion search "검색어"

# 결과 개수 제한
notion search "검색어" --limit 10
```

### 페이지 읽기

```bash
notion read <page_id>
```

### 페이지 생성

```bash
# 제목만
notion create --parent <parent_id> --title "새 페이지"

# 제목 + 내용
notion create --parent <parent_id> --title "새 페이지" --content "첫 문단"
```

### 내용 추가

```bash
notion append <page_id> "추가할 내용"
```

### 페이지 수정

```bash
# 제목 변경
notion update <page_id> --title "새 제목"

# 아이콘 변경
notion update <page_id> --icon "🚀"

# 둘 다
notion update <page_id> --title "새 제목" --icon "🚀"
```

### 페이지 삭제

```bash
notion delete <page_id>
```

페이지를 휴지통으로 이동합니다 (아카이브).

### 데이터베이스 쿼리

```bash
# 전체 조회
notion query <database_id>

# 필터
notion query <database_id> --filter "Status=Done"
notion query <database_id> --filter "Name:title=테스트"
notion query <database_id> --filter "Active:checkbox=true"

# 정렬
notion query <database_id> --sort "Created" --direction asc

# 개수 제한
notion query <database_id> --limit 10
```

**필터 형식:**
- `PropertyName=value` (기본: rich_text)
- `PropertyName:type=value`

**지원 타입:** `title`, `rich_text`, `select`, `checkbox`, `number`

### 공통 옵션

```bash
# 타임아웃 (기본: 30초)
notion --timeout 60 search "검색어"

# 버전
notion --version

# 도움말
notion --help
notion <command> --help
```

## Notion Integration 설정

1. [Notion Integrations](https://notion.so/my-integrations) 접속
2. "New integration" 클릭
3. 이름 입력 후 생성
4. "Internal Integration Token" 복사
5. **중요**: 접근할 페이지에서 Share → Integration 추가!

## 프로젝트 구조

```
notion-cli-rs/
├── Cargo.toml          # 의존성 설정
├── src/main.rs         # 전체 소스
├── docs/
│   ├── ARCHITECTURE.md # 코드 구조
│   ├── CARGO.md        # Cargo 가이드
│   └── API_COMPARISON.md # Notion API 분석
├── .env.example        # 환경변수 예시
└── .gitignore
```

## 특징

- **Rate Limit 자동 처리**: 429 응답 시 자동 재시도 (최대 3회)
- **페이지네이션 자동 처리**: 대량 데이터 자동 수집
- **UUID 유연한 입력**: 하이픈 있든 없든 모두 지원
- **컬러 출력**: 터미널 가독성 향상

## API 버전

Notion API `2025-09-03` 사용 (최신)

## 라이선스

MIT
