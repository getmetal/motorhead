import readline from "readline";
import * as dotenv from "dotenv"
dotenv.config()

import chalk from "chalk";
import { CallbackManager } from "langchain/callbacks";
import { ConversationChain } from "langchain/chains";
import { ChatOpenAI } from "langchain/chat_models/openai";
import {
  ChatPromptTemplate,
  HumanMessagePromptTemplate,
  SystemMessagePromptTemplate,
  MessagesPlaceholder,
} from "langchain/prompts";
import { MotorheadMemory } from "langchain/memory";

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

export const run = async () => {
  const chat = new ChatOpenAI({
    temperature: 0,
    streaming: true,
    callbackManager: CallbackManager.fromHandlers({
      async handleLLMNewToken(token) {
        process.stdout.write(chalk.green(token));
      },
    }),
  });

  const memory = new MotorheadMemory({
    returnMessages: true,
    memoryKey: "history",
    sessionId: "ozzy6666",
    motorheadURL: "http://localhost:8080"
  });
  await memory.init(); // loads previous state from Motorhead 🤘
  let context = "";

  if (memory.context) {
    context = `
      Here's previous context: ${memory.context}`;
  }

  const chatPrompt = ChatPromptTemplate.fromPromptMessages([
    SystemMessagePromptTemplate.fromTemplate(
      `The following is a friendly conversation between a human and an AI. The AI is talkative and provides lots of specific details from its context. If the AI does not know the answer to a question, it truthfully says it does not know.${context}`
    ),
    new MessagesPlaceholder("history"),
    HumanMessagePromptTemplate.fromTemplate("{input}"),
  ]);

  const chain = new ConversationChain({
    memory,
    prompt: chatPrompt,
    llm: chat,
  });

  const postToBash = async () => {
    console.log('\n')
    rl.question(chalk.green(`\n`), async function(answer) {
      const res = await chain.call({ input: answer });
      await postToBash(res.response);
    });
  };

  rl.question(
    chalk.blue(`\nMotorhead 🤘chat start\n`),
    async function(answer) {
      const res = await chain.call({ input: answer });
      await postToBash(res.response);
    }
  );
};

run();
