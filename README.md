# Motörhead

Motörhead is memory and information retrival server.

## Config

- `WINDOW_SIZE` - Number of max messages returned by the server. When this number is reached a job is triggered to halve it.
- `WINDOW_REDUCE_METHOD` - `summarization|buffer`
    - `summarization` - Once the `WINDOW_SIZE` is reached 1/4 of the window is summarized incrementally into the existing session summary.
    - `buffer`(default) - Memory only goes as far as the `WINDOW_SIZE`. Beyond that no messages are returned.
- `OPENAI_API_KEY` - Number of max messages returned by the server. When this number is reached a job is triggered to halve it. Required if `summarization` is the `WINDOW_REDUCE_METHOD`.

## Examples

- Check out our [Chat JS Example](examples/chat-js/)
