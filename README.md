# notion-cli-rs

Rust로 작성된 심플한 Notion CLI 도구.

## 설치

### 사전 요구사항
- Rust (1.70+)
- Notion Integration Token ([여기서 발급](https://notion.so/my-integrations))

### 빌드

```bash
# 개발 빌드
cargo build

# 릴리스 빌드 (최적화)
cargo build --release
```

빌드 결과물:
- 개발: `target/debug/notion`
- 릴리스: `target/release/notion`

### 전역 설치 (선택)

```bash
cargo install --path .
```

## 설정

1. `.env.example`을 `.env`로 복사:
```bash
cp .env.example .env
```

2. `.env` 파일에 API 키 설정:
```
NOTION_API_KEY=ntn_xxxxx
```

또는 환경변수로 직접 설정:
```bash
export NOTION_API_KEY=ntn_xxxxx
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

# 예시 (UUID 형식 또는 하이픈 없이 모두 가능)
notion read 12345678-1234-1234-1234-123456789abc
notion read 123456781234123412341234567890ab
```

### 페이지 생성
```bash
# 제목만
notion create --parent <parent_page_id> --title "새 페이지"

# 제목 + 내용
notion create --parent <parent_page_id> --title "새 페이지" --content "첫 번째 문단"
```

### 내용 추가
```bash
notion append <page_id> "추가할 내용"
```

### 옵션
```bash
# 타임아웃 설정 (기본: 30초)
notion --timeout 60 search "검색어"

# 버전 확인
notion --version

# 도움말
notion --help
```

## Notion Integration 설정

1. [Notion Integrations](https://notion.so/my-integrations) 접속
2. "New integration" 클릭
3. 이름 설정 후 생성
4. "Internal Integration Token" 복사
5. **중요**: 접근할 페이지에서 "Share" → Integration 추가 필요!

## 프로젝트 구조

```
notion-cli-rs/
├── Cargo.toml      # 의존성 및 프로젝트 설정
├── Cargo.lock      # 의존성 잠금 파일
├── src/
│   └── main.rs     # 전체 소스 코드 (단일 파일)
├── .env.example    # 환경 변수 예시
├── .env            # 실제 환경 변수 (gitignore)
└── .gitignore
```

## 아키텍처

자세한 내용은 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) 참조.

## 라이선스

MIT
