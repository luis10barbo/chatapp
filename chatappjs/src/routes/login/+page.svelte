<script lang="ts">
  import "./style.css";
  import { PUBLIC_URL_BACKEND } from "$env/static/public";
  import { getJson, postJson } from "../../utils/requests";
  import { goto } from "$app/navigation";
  const operacoes = {
    LOGIN: "LOGIN",
    REGISTRO: "REGISTRO",
  } as const;
  type Operacoes = keyof typeof operacoes;

  let usuario = "";
  let senha = "";
  let erro: string | undefined = undefined;
  let mensagem: string | undefined = undefined;
  let operacaoAtual: Operacoes = operacoes.LOGIN;
  function limparInput() {
    return;
    usuario = "";
    senha = "";
  }
  function setarMensagem(msg: string) {
    mensagem = msg;
    erro = undefined;
  }
  function setarErro(msg: string) {
    mensagem = undefined;
    erro = msg;
  }
  function limparMensagens() {
    mensagem = "";
    erro = "";
  }
  function processarDados() {
    if (operacaoAtual === operacoes.LOGIN) logar();
    else if (operacaoAtual === operacoes.REGISTRO) registrar();
  }
  async function logar() {
    limparMensagens();
    postJson(
      window.location.protocol + "//" + PUBLIC_URL_BACKEND + "/user/login",
      {
        usuario,
        senha,
      }
    ).then(async (res) => {
      if (res.status === 200) {
        setarMensagem("Login bem sucedido! Redirecionando...");
        window.location.replace("/");
      } else if (res.status === 401 || res.status === 404) {
        setarErro("Falha ao logar: " + (await res.text()));
      }
    });

    limparInput();
  }
  async function registrar() {
    limparMensagens();
    await postJson(
      window.location.protocol + "//" + PUBLIC_URL_BACKEND + "/user/registrar",
      {
        usuario,
        senha,
      }
    ).then(async (res) => {
      if (res.status === 200) {
        setarMensagem("Registro bem sucedido! Redirecionando...");
        window.location.replace("/");
      } else if (res.status === 404 || res.status === 409) {
        setarErro("Falha ao registrar: " + (await res.text()));
      }
    });
    limparInput();
  }
</script>

<div id="page">
  <div id="container-auth">
    {#if erro}
      <div class="caixa-mensagem caixa-erro">
        {erro}
      </div>
    {:else if mensagem}
      <div class="caixa-mensagem caixa-ok">
        {mensagem}
      </div>
    {/if}
    {#if operacaoAtual === operacoes.LOGIN}
      <p id="container-auth-title">Entrar</p>
    {:else if operacaoAtual === operacoes.REGISTRO}
      <p id="container-auth-title">Registrar</p>
    {/if}

    <section id="auth-input-holder">
      <label class="input-label">Usuario</label>
      <input
        bind:value={usuario}
        on:keydown={(e) => {
          if (e.key === "Enter") processarDados();
        }}
      />
      <label class="input-label">Senha</label>
      <input
        type="password"
        bind:value={senha}
        on:keydown={(e) => {
          if (e.key === "Enter") processarDados();
        }}
      />
    </section>

    <footer id="container-auth-footer">
      {#if operacaoAtual === operacoes.LOGIN}
        <p class="full-width">
          Não tem uma conta? <button
            class="text-button"
            on:click={() => {
              operacaoAtual = operacoes.REGISTRO;
            }}>Cadastrar.</button
          >
        </p>
        <button
          id="send-button"
          on:click={() => {
            processarDados();
          }}>Entrar</button
        >
      {:else if operacaoAtual === operacoes.REGISTRO}
        <p class="full-width">
          Já está cadastrado? <button
            class="text-button"
            on:click={() => {
              operacaoAtual = operacoes.LOGIN;
            }}>Logar.</button
          >
        </p>
        <button
          id="send-button"
          on:click={() => {
            processarDados();
          }}>Registrar</button
        >
      {/if}
    </footer>
  </div>
</div>
