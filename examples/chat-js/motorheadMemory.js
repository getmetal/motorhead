import { BaseChatMemory } from "langchain/memory";

const MOTORHEAD_URL = "http://localhost:8080";

export default class MotorheadMemory extends BaseChatMemory {
  humanPrefix = "Human";
  aiPrefix = "AI";
  memoryKey = "history";
  sessionId = "test123";

  constructor(fields) {
    super({
      returnMessages: fields?.returnMessages ?? false,
      inputKey: fields?.inputKey,
      outputKey: fields?.outputKey,
    });
    this.humanPrefix = fields?.humanPrefix ?? this.humanPrefix;
    this.aiPrefix = fields?.aiPrefix ?? this.aiPrefix;
    this.memoryKey = fields?.memoryKey ?? this.memoryKey;
    this.sessionId = fields?.sessionId ?? this.sessionId;
  }

  async init() {
    const res = await fetch(
      `${MOTORHEAD_URL}/sessions/${this.sessionId}/memory`,
      {
        headers: {
          "Content-Type": "application/json",
        },
      }
    );

    const { messages = [], context = "NONE" } = await res.json();

    messages.forEach((message) => {
      const isAIMessage = message.startsWith("AI:");
      if (isAIMessage) {
        this.chatHistory.addAIChatMessage(message.substring(3));
      } else {
        this.chatHistory.addUserMessage(message.substring(5));
      }
    });

    if (context && context !== "NONE") {
      this.context = context;
    }
  }

  async loadMemoryVariables() {
    const result = {
      [this.memoryKey]: this.chatHistory.messages,
    };
    return result;
  }

  async saveContext(inputValues, outputValues) {
    await fetch(`${MOTORHEAD_URL}/sessions/${this.sessionId}/memory`, {
      method: "POST",
      body: JSON.stringify({
        messages: [
          { message: `Human: ${inputValues.input}` },
          { message: `AI: ${outputValues.response}` },
        ],
      }),
      headers: {
        "Content-Type": "application/json",
      },
    });

    super.saveContext(inputValues, outputValues);
  }
}
