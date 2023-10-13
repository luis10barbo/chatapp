<script context="module" lang="ts">
  export type Chat = {
    chat_id: string;
    chat_name: string;
    chat_desc: string;
  };
</script>

<script lang="ts">
  import { PUBLIC_URL_BACKEND } from "$env/static/public";
  import { getJson } from "../utils/requests";
  import { selectChat, selectedChat } from "./+page.svelte";
  import ChatCard from "./ChatCard.svelte";
  let chats: Chat[] | undefined = undefined;
  async function getCards() {
    const res = await getJson(
      location.protocol + "//" + PUBLIC_URL_BACKEND + "/chat/"
    );
    if (res.status !== 200) {
      console.error("Erro adquirindo chats");
    }
    chats = JSON.parse(await res.text()) as Chat[];
  }
  getCards();
</script>

<section id="chats-holder">
  <div id="chat-search-holder">
    <input placeholder="Pesquisar uma conversa..." />
    <button>Pesquisar</button>
  </div>

  {#if chats}
    {#each chats as chat}
      <ChatCard
        {chat}
        click={() => {
          selectChat(chat.chat_id);
        }}
      />
    {/each}
  {/if}
</section>
