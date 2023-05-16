import chalk from "chalk";
import { Configuration, OpenAIApi } from "openai";
import readline from "readline";
import * as dotenv from "dotenv"
import { fetchMessages, postMessages, retrieval, streamResponse } from "./helpers.js";
dotenv.config()

const MODEL = 'gpt-3.5-turbo'
const ASSISTANT = 'assistant'
const USER = 'user'

const configuration = new Configuration({ apiKey: process.env.OPENAI_API_KEY });
const openai = new OpenAIApi(configuration);

const generateSystemMessage = (context, longTermMemory = []) => {
  const longTermMemoryContext = longTermMemory.filter(({ role }) => role === ASSISTANT).map(({ content }) => content)[0];

  return `The following is a friendly conversation between a human and an AI. The AI is talkative and provides lots of specific details from its context. If the AI does not know the answer to a question, it truthfully says it does not know.\n${context}\n${longTermMemoryContext}`
}

const promptModel = async (query, messages, context) => {
  const systemMessageContent = generateSystemMessage(context)
  const systemMessageObj = { role: 'system', content: systemMessageContent }
  const completion = await openai.createChatCompletion({
    model: MODEL,
    messages: [systemMessageObj, ...messages, { role: 'user', content: query }],
    stream: true,
  }, { responseType: 'stream' })
    .catch(err => {
      console.log('ERROR', err.toJSON())
      throw new Error('openai failed');
    });

  return streamResponse(completion);
}


const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});


const chat = async (context, messages) => {
  rl.question(
    "\n",
    async function(query) {
      const response = await promptModel(query, messages, context);
      const newMessages = [{ role: USER, content: query }, { role: ASSISTANT, content: response }];

      await postMessages(newMessages);

      messages.push(...newMessages);

      await chat(context, messages);
    }
  );
};

export const run = async () => {
  const { context, messages } = await fetchMessages()
    .catch(err => {
      console.log(`Error starting up:`, err.message)
      console.log(chalk.bgCyanBright(`\nIs Motorhead Running?\n`))
      process.exit(1)
    });

  console.log(chalk.blue(`\nMotorhead 🤘chat start\n`))
  chat(context, messages.reverse());
};


run();
