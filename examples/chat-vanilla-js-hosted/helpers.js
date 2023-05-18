import chalk from "chalk";
import * as dotenv from "dotenv"
dotenv.config()

const MOTORHEAD_URL = "https://api.getmetal.io/v1/motorhead"
const API_KEY = process.env.METAL_API_KEY
const CLIENT_ID = process.env.METAL_CLIENT_ID

export const fetchMessages = () => fetch(`${MOTORHEAD_URL}/sessions/ozzy/memory`, {
  method: "GET",
  headers: {
    'Content-Type': 'application/json',
    'x-metal-api-key': API_KEY,
    'x-metal-client-id': CLIENT_ID,
  },
}).then(res => res.json())
  .then(res => res.data)

export const postMessages = (messages) => fetch(`${MOTORHEAD_URL}/sessions/ozzy/memory`, {
  method: "POST",
  headers: {
    'Content-Type': 'application/json',
    'x-metal-api-key': API_KEY,
    'x-metal-client-id': CLIENT_ID,
  },
  body: JSON.stringify({ messages })
}).then(res => res.json())
  .then(res => res.data)

export const retrieval = async (query) => {
  if (!query) return Promise.resolve([])

  return fetch(`${MOTORHEAD_URL}/sessions/ozzy/retrieval`, {
    method: "POST",
    headers: {
      'Content-Type': 'application/json',
      'x-metal-api-key': API_KEY,
      'x-metal-client-id': CLIENT_ID,
    },
    body: JSON.stringify({ text: query })
  }).then(res => res.json())
    .then(res => res.data)
}

export function streamResponse(completion) {
  return new Promise((resolve) => {
    let result = "";
    completion.data.on("data", (data) => {
      const lines = data
        ?.toString()
        ?.split("\n")
        .filter((line) => line.trim() !== "");
      for (const line of lines) {
        const message = line.replace(/^data: /, "");
        if (message == "[DONE]") {
          process.stdout.write(chalk.green('\n'));
          resolve(result);
        } else {
          let token;
          try {
            token = JSON.parse(message)?.choices?.[0]?.delta.content;
          } catch (err) {
            // console.log("ERROR", err);
          }

          if (token) {
            result += token;
            process.stdout.write(chalk.green(token));
          }
        }
      }
    });
  });
}
