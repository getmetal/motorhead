import chalk from "chalk";
const MOTORHEAD_URL = "http://localhost:8080"

export const fetchMessages = () => fetch(`${MOTORHEAD_URL}/sessions/ozzy/memory`, {
  method: "GET",
  headers: { 'Content-Type': 'application/json' },
}).then(res => res.json())

export const postMessages = (messages) => fetch(`${MOTORHEAD_URL}/sessions/ozzy/memory`, {
  method: "POST",
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ messages })
}).then(res => res.json())

export const retrieval = async (query) => {
  if (!query) return Promise.resolve([])

  return fetch(`${MOTORHEAD_URL}/sessions/ozzy/retrieval`, {
    method: "POST",
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ text: query })
  }).then(res => res.json())
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
