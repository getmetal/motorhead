<h1 align="center" >
Motörhead
</h1>
<p align="center">
    <a href="https://github.com/getmetal/motorhead/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/getmetal/motorhead?style=flat&label=license&logo=github&color=4f46e5&logoColor=fff" alt="License" />
    </a>
  <a href="https://twitter.com/Metal_io">
    <img src="https://img.shields.io/badge/metal-message?style=flat&logo=twitter&color=4f46e5&logoColor=#4f46e5" alt="Metal" style="margin-right:3px" />
  </a>
  <a href="https://discord.gg/GHY3Y8tU3J">
    <img src="https://dcbadge.vercel.app/api/server/GHY3Y8tU3J?compact=true&style=flat" alt="License" />
  </a>
</p>

Motörhead is a memory and information retrieval server for LLMs.

## Why use Motörhead?

When building chat applications using LLMs, memory handling is something that  has to be built every time. Motörhead is a server to assist with that process. It provides 3 simple APIs:

- GET `/sessions/:id/memory` returns messages up to `MAX_WINDOW_SIZE`.
```json
{
    "messages": [
        "AI: Motörhead had many members over the years, but some of the most notable ones include:\n\n- Phil \"Philthy Animal\" Taylor: drummer from 1975-1984 and 1987-1992\n- \"Fast\" Eddie Clarke: guitarist from 1976-1982\n- Brian \"Robbo\" Robertson: guitarist from 1982-1983\n- Würzel: guitarist from 1984-1995\n- Mikkey Dee: drummer from 1992-2015\n\nThese are just a few of the many musicians who played with Motörhead over the years. However, Lemmy Kilmister was the only member who was with the band from its inception until his death in 2015.",
        "Human: What are the other members names?",
        "AI: Motörhead was formed in London, England in 1975. The band was founded by bassist and vocalist Ian \"Lemmy\" Kilmister, who had previously played with the space rock band Hawkwind. The original lineup of Motörhead also included guitarist Larry Wallis and drummer Lucas Fox. Over the years, the band's lineup changed several times, with Lemmy being the only constant member. Despite their English origins, Motörhead became a global phenomenon, with a dedicated fanbase around the world.",
        "Human: Where is Motörhead from? ",
        "AI: Motörhead had several popular songs throughout their career, but their biggest hit was likely \"Ace of Spades\". The song was released in 1980 and quickly became a fan favorite, thanks to its fast-paced, hard-hitting sound and catchy chorus. \"Ace of Spades\" is often cited as one of the greatest heavy metal songs of all time and has been covered by numerous other artists over the years. Other popular songs by Motörhead include \"Overkill\", \"Bomber\", \"Iron Fist\", and \"Killed by Death\".",
        "Human: What was Motörhead's biggest song?",
        "AI: The lead singer of Motörhead was Lemmy Kilmister. He was also the bassist and primary songwriter for the band. Lemmy was known for his distinctive gravelly voice, his aggressive bass playing style, and his iconic handlebar mustache. He passed away in 2015 at the age of 70, but his legacy as a rock and roll icon lives on."
    ],
    "context": "The human inquires about cool neighborhoods and the AI lists several examples from around the world. The conversation then transitions to discussing the increasingly popular neighborhood of Williamsburg in Brooklyn, which has a trendy and hipster vibe. The AI also mentions similar neighborhoods in Paris. The conversation then abruptly changes as the human asks who the lead singer of Motörhead was. The AI answers with information about Lemmy Kilmister, the bassist and primary songwriter for the band. The human then asks about famous techno clubs in Berlin, and the AI lists several examples with unique atmospheres and cutting-edge music. The AI explains that techno originated in Detroit in the mid-1980s and quickly gained popularity as a symbol of freedom and unity. The genre has since spread globally and continues to evolve and innovate in new sub-genres and styles."
}
```

- POST `/sessions/:id/memory` - you can send multiple messages to Motorhead to store.

```bash
curl --location 'localhost:8080/sessions/${SESSION_ID}/memory' \
--header 'Content-Type: application/json' \
--data '{
    "messages": [{ "message": "Human: ping" }, { "message": "AI: pong" }]
}'
```
- DELETE `/sessions/:id/memory` - deletes the session's message list.

A max `window_size` is set for the LLM to keep track of the conversation. Once that max is hit, Motörhead will process (`window_size  / 2` messages) and summarize them. Subsequent summaries, as the messages grow, are incremental.

## Config

- `MAX_WINDOW_SIZE` (default:10) - Number of max messages returned by the server. When this number is reached, a job is triggered to halve it.
- `OPENAI_API_KEY` (required)- Number of max messages returned by the server. When this number is reached, a job is triggered to halve it.

## How to run

With Docker:
```bash

docker-compose build && docker-compose up

```

## Examples

- Check out our [Chat JS Example](examples/chat-js/)
