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
      if (message.role == 'AI') {
        this.chatHistory.addAIChatMessage(message.content);
      } else {
        this.chatHistory.addUserMessage(message.content);
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
          { role: 'Human', content: `${inputValues.input}` },
          { role: 'AI', content: `${outputValues.response}` },
        ],
      }),
      headers: {
        "Content-Type": "application/json",
      },
    });

    super.saveContext(inputValues, outputValues);
  }
}
