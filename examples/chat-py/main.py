import sys
import asyncio
from typing import Any, Union, Dict, List

from langchain.callbacks import CallbackManager, BaseCallbackManager, BaseCallbackHandler
from langchain.chains import ConversationChain
from langchain.chat_models.openai import ChatOpenAI
from langchain.prompts import (
    ChatPromptTemplate,
    HumanMessagePromptTemplate,
    SystemMessagePromptTemplate,
    MessagesPlaceholder,
)
from dotenv import load_dotenv
from langchain.memory.motorhead_memory import MotorheadMemory
from langchain.schema import AgentFinish, AgentAction, LLMResult
from termcolor import colored

load_dotenv()


class CustomCallbackManager(BaseCallbackManager):

    def add_handler(self, callback: BaseCallbackHandler) -> None:
        pass

    def remove_handler(self, handler: BaseCallbackHandler) -> None:
        pass

    def set_handlers(self, handlers: List[BaseCallbackHandler]) -> None:
        pass

    def on_llm_start(self, serialized: Dict[str, Any], prompts: List[str], **kwargs: Any) -> Any:
        pass

    def on_llm_new_token(self, token: str, **kwargs: Any) -> Any:
        return sys.stdout.write(colored(token, "green"))

    def on_llm_end(self, response: LLMResult, **kwargs: Any) -> Any:
        pass

    def on_llm_error(self, error: Union[Exception, KeyboardInterrupt], **kwargs: Any) -> Any:
        pass

    def on_chain_start(self, serialized: Dict[str, Any], inputs: Dict[str, Any], **kwargs: Any) -> Any:
        pass

    def on_chain_end(self, outputs: Dict[str, Any], **kwargs: Any) -> Any:
        pass

    def on_chain_error(self, error: Union[Exception, KeyboardInterrupt], **kwargs: Any) -> Any:
        pass

    def on_tool_start(self, serialized: Dict[str, Any], input_str: str, **kwargs: Any) -> Any:
        pass

    def on_tool_end(self, output: str, **kwargs: Any) -> Any:
        pass

    def on_tool_error(self, error: Union[Exception, KeyboardInterrupt], **kwargs: Any) -> Any:
        pass

    def on_text(self, text: str, **kwargs: Any) -> Any:
        pass

    def on_agent_action(self, action: AgentAction, **kwargs: Any) -> Any:
        pass

    def on_agent_finish(self, finish: AgentFinish, **kwargs: Any) -> Any:
        pass


async def run():
    cb_manager = CallbackManager(handlers=[CustomCallbackManager()])
    chat = ChatOpenAI(
        temperature=0,
        streaming=True,
        callback_manager=cb_manager
    )

    memory = MotorheadMemory(
        return_messages=True,
        memory_key="history",
        session_id="davemustaine666",
        url="http://localhost:8080",
    )
    await memory.init()

    context = ""
    if memory.context:
        context = f"\nHere's previous context: {memory.context}"

    chat_prompt = ChatPromptTemplate.from_messages(
        [
            SystemMessagePromptTemplate.from_template(
                f"The following is a friendly conversation between a human and an AI. The AI is talkative and "
                f"provides lots of specific details from its context. If the AI does not know the answer to a "
                f"question, it truthfully says it does not know. {context}"
            ),
            MessagesPlaceholder(variable_name="history"),
            HumanMessagePromptTemplate.from_template("{input}"),
        ]
    )
    chain = ConversationChain(memory=memory, prompt=chat_prompt, llm=chat)

    def post_to_bash():
        while True:
            answer_i = input(colored("", "green"))
            if not answer_i:
                continue
            response_i = chain.run(answer_i)
            print(colored(response_i, "green"))

    print(colored("\nMotorhead 🤘chat start\n", "blue"))
    answer = input(colored("", "green"))
    response = chain.run(answer)
    print(colored(response, "green"))
    post_to_bash()


if __name__ == "__main__":
    try:
        asyncio.run(run())
    except KeyboardInterrupt as kie:
        print(colored("\nI see you have chosen to end the conversation with me 💔. Good bye!", "yellow"))
