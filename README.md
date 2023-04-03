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
        {
            "role": "AI",
            "content": "Electronic music and salsa are two very different genres of music, and the way people dance to them is also quite different.\n\nSalsa is a Latin dance style that originated in the Caribbean and is characterized by its lively, syncopated rhythm and intricate footwork. Salsa dancing often involves partners dancing in close proximity, with a lot of hip and body movement. The dance is often very sensual and requires a lot of coordination between partners.\n\nElectronic music, on the other hand, is often associated with more free-form, individualistic dancing. While there are certainly partner dances that can be done to electronic music, such as the \"shuffle\" or \"jacking\" styles, many people prefer to dance on their own and let the music guide their movements. Electronic music is often characterized by a strong, steady beat that can be danced to in a variety of ways, from energetic jumping and bouncing to more fluid, flowing movements.\n\nIn summary, while both salsa and electronic music can be great for dancing, they are very different styles of music with different dance styles and techniques."
        },
        {
            "role": "Human",
            "content": "how does it compare to salsa?"
        },
        {
            "role": "AI",
            "content": "Electronic music is a broad genre that encompasses many different styles, so there is no one \"right\" way to dance to it. However, there are some general tips that can help you get started:\n\n1. Feel the beat: Electronic music is often characterized by a strong, steady beat. Try to feel the rhythm of the music and let it guide your movements.\n\n2. Move your body: Electronic music is often associated with energetic, full-body movements. Don't be afraid to let loose and move your arms, legs, and hips.\n\n3. Experiment with different styles: Depending on the style of electronic music you're listening to, there may be different dance styles that are associated with it. For example, techno and house music are often associated with \"rave\" or \"club\" dancing, while breakbeat and drum and bass are often associated with more athletic, breakdancing-style movements.\n\n4. Watch others: If you're new to electronic music, it can be helpful to watch others dance to get a sense of the style and rhythm. You can also find many tutorials and instructional videos online that can help you learn specific dance moves.\n\nRemember, the most important thing is to have fun and let the music move you!"
        },
        {
            "role": "Human",
            "content": "how do you dance electronic music?"
        },
        {
            "role": "AI",
            "content": "Colombia has a vibrant electronic music scene, and there are many talented DJs and producers who have gained international recognition. Here are a few famous DJs from Colombia:\n\n1. Erick Morillo: Born in New York City to Colombian parents, Erick Morillo is a DJ and producer who has had a long and successful career in the electronic music industry. He is best known for his house music tracks and has won several awards for his work.\n\n2. Kali Uchis: Although not strictly a DJ, Kali Uchis is a Colombian-American singer and songwriter who has gained a large following for her unique blend of R&B, soul, and pop music. She has collaborated with many famous musicians, including Tyler, The Creator and Gorillaz.\n\n3. DJ Fresh: DJ Fresh is a Colombian DJ and producer who is known for his work in the drum and bass genre. He has released several albums and has won numerous awards for his music.\n\n4. Mario Ochoa: Mario Ochoa is a DJ and producer from Medellín, Colombia, who is known for his techno and tech-house tracks. He has released music on many famous labels, including Toolroom and 100% Pure.\n\nThese are just a few examples of the many talented DJs and producers from Colombia."
        },
        {
            "role": "Human",
            "content": "What are some famous djs from Colombia?"
        },
        {
            "role": "AI",
            "content": "Baum opened its doors in 2014 and has quickly become one of the most popular clubs for electronic music in Bogotá. It is known for its impressive sound system and its focus on underground techno and house music.\n\nVideo Club, on the other hand, has been around for much longer. It first opened in 1999 and has since become an institution in the Bogotá nightlife scene. It is known for its eclectic music selection, which includes everything from electronic music to rock and pop. Over the years, Video Club has hosted many famous DJs and musicians, including Daft Punk, Chemical Brothers, and LCD Soundsystem."
        }
    ],
    "context": "The conversation covers topics such as clubs for electronic music in Bogotá, popular tourist attractions in the city, and general information about Colombia. The AI provides information about popular electronic music clubs such as Baum and Video Club, as well as electronic music festivals that take place in Bogotá. The AI also recommends tourist attractions such as La Candelaria, Monserrate and the Salt Cathedral of Zipaquirá, and provides general information about Colombia's diverse culture, landscape and wildlife."
}
```

- POST `/sessions/:id/memory` - you can send multiple messages to Motorhead to store.

```bash
curl --location 'localhost:8080/sessions/${SESSION_ID}/memory' \
--header 'Content-Type: application/json' \
--data '{
    "messages": [{ "role": "Human", "content": "ping" }, { "role": "AI", "content": "pong" }]
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
