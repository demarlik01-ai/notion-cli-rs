# 아키텍처

## 개요

단일 파일(`src/main.rs`) 구조의 심플한 CLI 애플리케이션.
Notion REST API를 래핑하여 터미널에서 사용 가능하게 함.

## 구조

```
┌─────────────────────────────────────────────────────────┐
│                        main()                           │
│  - CLI 파싱 (clap)                                      │
│  - API 키 로드                                          │
│  - NotionClient 초기화                                  │
│  - 명령어 라우팅                                        │
└─────────────────────────────────────────────────────────┘
                           │
           ┌───────────────┼───────────────┐
           ▼               ▼               ▼
    ┌────────────┐  ┌────────────┐  ┌────────────┐
    │   Search   │  │    Read    │  │   Create   │
    │  handler   │  │  handler   │  │  handler   │
    └────────────┘  └────────────┘  └────────────┘
           │               │               │
           └───────────────┴───────────────┘
                           │
                           ▼
           ┌───────────────────────────────┐
           │        NotionClient           │
           │  - search()                   │
           │  - get_page()                 │
           │  - get_blocks()               │
           │  - create_page()              │
           │  - append_blocks()            │
           └───────────────────────────────┘
                           │
                           ▼
           ┌───────────────────────────────┐
           │      Notion REST API          │
           │   https://api.notion.com/v1   │
           └───────────────────────────────┘
```

## 핵심 컴포넌트

### 1. CLI (clap derive)

```rust
struct Cli {
    command: Commands,
    timeout: u64,  // 글로벌 옵션
}

enum Commands {
    Search { query, limit },
    Read { page_id },
    Create { parent, title, content },
    Append { page_id, content },
}
```

### 2. NotionClient

HTTP 클라이언트 래퍼. 모든 API 호출을 담당.

- **인증**: Bearer 토큰 (NOTION_API_KEY)
- **버전**: Notion-Version 헤더 (기본 2022-06-28)
- **HTTP**: reqwest blocking 클라이언트

### 3. 페이지네이션

`search()`와 `get_blocks()`는 자동 페이지네이션 지원:

```rust
loop {
    let response = api_call(start_cursor)?;
    results.extend(response.results);
    
    if !response.has_more { break; }
    start_cursor = response.next_cursor;
}
```

### 4. 헬퍼 함수

- `normalize_page_id()`: 다양한 형식의 페이지 ID를 UUID 형식으로 정규화
- `extract_title()`: 복잡한 Notion 객체에서 제목 추출
- `extract_rich_text()`: 블록에서 텍스트 추출
- `print_block()`: 블록 타입별 출력 포맷팅

## 데이터 흐름

### Search

```
사용자 입력 → POST /search → 결과 파싱 → 제목/ID 출력
```

### Read

```
페이지 ID → GET /pages/{id} → 메타데이터
         → GET /blocks/{id}/children → 블록 목록 → 포맷팅 출력
```

### Create

```
부모 ID + 제목 + 내용 → POST /pages → 생성된 페이지 ID/URL 출력
```

### Append

```
페이지 ID + 내용 → PATCH /blocks/{id}/children → 완료 메시지
```

## 에러 처리

- `anyhow`: 에러 체이닝 및 컨텍스트 추가
- 모든 API 호출에 `.context()` 적용
- `main()`에서 통합 에러 핸들링 (exit code 1)

## 출력 포맷

- `colored` 크레이트로 터미널 색상 지원
- 성공: ✓ (녹색)
- 실패: ✗ (빨강)
- 정보: 파랑/시안
- 메타데이터: dimmed

## 제한사항

- Blocking HTTP (async 미사용) - CLI 특성상 충분
- 단일 파일 구조 - 규모가 작아서 충분
- Rich text 편집 미지원 - plain text만 생성/추가 가능
- 데이터베이스 아이템 생성 미지원 - 페이지만 가능
