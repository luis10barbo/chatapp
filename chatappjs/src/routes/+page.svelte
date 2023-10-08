<script lang="ts">
  import CardMensagem from "./CardMensagem.svelte";
  import type { Mensagem } from "./CardMensagem.svelte";
  // import { WebSocket } from "ws";
  import { browser } from "$app/environment";
  import "./style.css";
  import { onMount } from "svelte";
  import ContainerChat from "./ContainerChat.svelte";
  const meuId = 0;
  let mensagens: Mensagem[];
  $: mensagens = [
    { horario: "10:50", mensagem: "Oi", nome: "Luis", id: 0 },
    { horario: "10:50", mensagem: "Eae mano", nome: "Jorge", id: 1 },
    { horario: "10:50", mensagem: "Falaaa", nome: "Luis", id: 0 },
  ];

  $: teste = "";
  const func = () => {
    console.log("teste");
    teste = "teste";
  };
</script>

<div id="page">
  <section id="chats">
    <header id="chats-header" class="section-header">
      <p>Conversas {teste}</p>
    </header>
    <section id="chats-holder">
      <div id="chat-search-holder">
        <input placeholder="Pesquisar uma conversa..." />
        <button on:click={() => func()}>Pesquisar</button>
      </div>
      <button class="chat-card">
        <header class="chat-card-header">
          <p class="chat-card-name">Titulo</p>
        </header>
        <footer class="chat-card-footer">
          <p class="chat-card-time">10:45</p>
          <p class="chat-card-status">
            <span class="chat-card-status-name">Luis</span>
            <span class="chat-card-status-msg">teste teste</span>
          </p>
        </footer>
      </button>
    </section>
    <footer id="chats-footer" class="section-footer">
      <button>Contatos</button>
      <button>Perfil</button>
      <button>Sair</button>
    </footer>
  </section>
  <ContainerChat idChat="d9b49810-a1cb-440a-9e66-c293aa61d4d9" />
</div>
