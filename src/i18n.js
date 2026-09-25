const PT='pt', EN='en';
let lang=localStorage.getItem('veyra.language')||((navigator.language||'pt').toLowerCase().startsWith('pt')?PT:EN);
const originals=new WeakMap(), attrOriginals=new WeakMap();
const map={
'Definições do perfil':'Settings for profile','Extensões Chromium deste perfil:':'Chromium extensions for this profile:','Início':'Home','Marcadores':'Bookmarks','Histórico':'History','Transferências':'Downloads','Extensões':'Extensions','Definições':'Settings','Favoritos':'Favorites','Pronto':'Ready','Nova aba':'New tab','Novo grupo':'New group','Novo perfil':'New profile','Perfis':'Profiles','Grupos de abas':'Tab groups','Menu Veyra':'Veyra Menu','Aparência':'Appearance','Motor de pesquisa':'Search engine','Dados de navegação':'Browsing data','Atualizações':'Updates','Sleeping tabs':'Sleeping tabs',
'Adicionar':'Add','Abrir':'Open','Remover':'Remove','Apagar':'Delete','Renomear':'Rename','Cor':'Color','Fechar aba':'Close tab','Duplicar aba':'Duplicate tab','Adormecer aba':'Sleep tab','Retirar do grupo':'Remove from group','Mover para grupo':'Move to group','Abrir pasta':'Open folder','Adicionar desempacotada':'Load unpacked','Desativar extensões':'Disable extensions','Ativar extensões':'Enable extensions','Limpar histórico':'Clear history','Limpar lista':'Clear list','Limpar concluídos':'Clear completed','Mostrar na pasta':'Show in folder',
'Alternar tema':'Toggle theme','Limpar dados deste perfil':'Clear this profile data','Perfil atual':'Current profile','Mudar para este perfil':'Switch to this profile','Gerir perfis':'Manage profiles','Adicionar aba atual':'Add current tab','Expandir':'Expand','Recolher':'Collapse','Verificar atualizações':'Check for updates','Instalar atualização':'Install update','A verificar atualizações…':'Checking for updates…','A transferir e instalar atualização…':'Downloading and installing update…',
'Navega do teu jeito.':'Browse your way.','Céu limpo':'Clear sky','TECNOLOGIA':'TECHNOLOGY','PRODUTIVIDADE':'PRODUCTIVITY','INSPIRAÇÃO':'INSPIRATION','PRIVACIDADE':'PRIVACY','O futuro da navegação está aqui':'The future of browsing is here','Mais rápido, mais seguro, mais teu.':'Faster, safer, more yours.','Menos ruído. Mais foco.':'Less noise. More focus.','Um browser leve pensado para trabalhar.':'A lightweight browser built for work.','Explora novos horizontes':'Explore new horizons','A web sem distrações desnecessárias.':'The web without unnecessary distractions.','A tua privacidade primeiro':'Your privacy first','Controlo local e navegação segura.':'Local control and secure browsing.',
'Procurar no Veyra ou escrever um endereço...':'Search Veyra or enter an address...','Voltar':'Back','Avançar':'Forward','Atualizar':'Reload','Adicionar aos marcadores':'Add bookmark','Ir':'Go','Perfil':'Profile','Menu':'Menu','Transferências':'Downloads','Proton Pass / Extensões':'Proton Pass / Extensions','Idioma da interface do Veyra.':'Veyra interface language.','A obter versão atual.':'Getting current version…'
};
Object.assign(map,{
'Os teus favoritos guardados localmente no Veyra.':'Your favorites stored locally in Veyra.','Páginas visitadas neste perfil.':'Pages visited in this profile.','Ficheiros transferidos pelo Veyra.':'Files downloaded by Veyra.','Acesso rápido às ferramentas do browser.':'Quick access to browser tools.','Ver e gerir favoritos.':'View and manage favorites.','Downloads e ficheiros concluídos.':'Downloads and completed files.','Proton Pass e extensões Chromium.':'Proton Pass and Chromium extensions.','Organizar abas por grupo e cor.':'Organize tabs by group and color.','Trocar e gerir perfis isolados.':'Switch and manage isolated profiles.','Pesquisa, aparência e sleeping tabs.':'Search, appearance and sleeping tabs.',
'Gestor de palavras-passe com autofill nas páginas deste perfil.':'Password manager with autofill on pages in this profile.','A verificar…':'Checking…','Instaladas':'Installed','O Proton Pass vem incluído. Podes adicionar outras extensões Chromium desempacacotadas.':'Proton Pass is included. You can add other unpacked Chromium extensions.','Isolamento por perfil':'Per-profile isolation','Cada perfil usa um diretório WebView2 próprio para cookies, logins e dados dos sites. Ativar/desativar extensões aplica-se apenas ao perfil atual.':'Each profile uses its own WebView2 directory for cookies, logins and site data. Enabling/disabling extensions only applies to the current profile.',
'Usado quando escreves palavras na barra de endereço.':'Used when you type words in the address bar.','Alterna entre a interface escura e clara neste perfil.':'Switch between dark and light interface in this profile.','Remove cookies, cache e armazenamento apenas deste perfil.':'Remove cookies, cache and storage only for this profile.','Descarrega abas inativas da memória e recarrega-as quando voltares.':'Unload inactive tabs from memory and reload them when you return.','AdBlock integrado':'Built-in AdBlock','Bloqueia pedidos de anúncios conhecidos diretamente no motor WebView2, sem extensão.':'Blocks known ad requests directly in WebView2, without an extension.','Desativar AdBlock':'Disable AdBlock','Ativar AdBlock':'Enable AdBlock','Nunca':'Never','5 minutos':'5 minutes','10 minutos':'10 minutes','15 minutos':'15 minutes','30 minutos':'30 minutes','1 hora':'1 hour','Separa trabalho, pessoal e outras sessões do browser.':'Separate work, personal and other browser sessions.','Dados, abas e logins separados':'Separate data, tabs and logins'
});Object.assign(map,{
'Ainda não tens grupos. Cria um e a aba atual entra automaticamente nele.':'You do not have any groups yet. Create one and the current tab will be added automatically.','Ainda não tens marcadores. Usa a estrela na barra de endereço.':'You do not have bookmarks yet. Use the star in the address bar.','O histórico está vazio.':'History is empty.','Ainda não existem transferências.':'There are no downloads yet.','Não existem extensões instaladas.':'No extensions are installed.'
});
function translateString(src){
 if(lang===PT)return src;
 const m=String(src).match(/^(\s*)(.*?)(\s*)$/s);const lead=m?.[1]||'',core=m?.[2]??String(src),tail=m?.[3]||'';
 let s=map[core]||core;
 s=s.replace(/^Definições do perfil (.+)\.$/,'Settings for profile $1.').replace(/^Extensões Chromium deste perfil: /,'Chromium extensions for this profile: ');
 s=s.replace(/^(\d+) abas$/,'$1 tabs').replace(/^(\d+) abas · Recolhido$/,'$1 tabs · Collapsed').replace(/^(\d+) abas · Expandido$/,'$1 tabs · Expanded');
 s=s.replace(/^Perfil: /,'Profile: ').replace(/^Perfil atual · /,'Current profile · ').replace(/^Ativa · /,'Active · ').replace(/^Desativada neste perfil · /,'Disabled in this profile · ');
 s=s.replace(/^Versão atual: /,'Current version: ').replace(/^Atualização v(.+) disponível \(atual: v(.+)\)\.$/,'Update v$1 available (current: v$2).').replace(/^Veyra v(.+) está atualizado\.$/,'Veyra v$1 is up to date.');
 s=s.replace(/^A transferir…$/,'Downloading…').replace(/^Concluído$/,'Completed').replace(/^Falhou$/,'Failed').replace(/^(\d+) bloqueados nesta sessão$/,'$1 blocked this session');
 return lead+s+tail;
}
function applyNode(root=document){
 const walker=document.createTreeWalker(root,NodeFilter.SHOW_TEXT);let n;
 while(n=walker.nextNode()){if(!n.parentElement||['SCRIPT','STYLE'].includes(n.parentElement.tagName))continue;if(!originals.has(n))originals.set(n,n.nodeValue);n.nodeValue=translateString(originals.get(n));}
 root.querySelectorAll?.('[placeholder],[title]').forEach(x=>{let data=attrOriginals.get(x);if(!data){data={placeholder:x.getAttribute('placeholder'),title:x.getAttribute('title')};attrOriginals.set(x,data);}if(data.placeholder!==null)x.setAttribute('placeholder',translateString(data.placeholder));if(data.title!==null)x.setAttribute('title',translateString(data.title));});
 document.documentElement.lang=lang===PT?'pt':'en';
}
export function getLanguage(){return lang;}
export function setLanguage(next){lang=next===EN?EN:PT;localStorage.setItem('veyra.language',lang);applyNode(document);window.dispatchEvent(new CustomEvent('veyra-language-changed',{detail:lang}));}
export function tr(src){return translateString(src);}
export function initI18n(){
 applyNode(document);
 new MutationObserver(ms=>ms.forEach(m=>m.addedNodes.forEach(n=>{
  if(n.nodeType===Node.TEXT_NODE){if(!originals.has(n))originals.set(n,n.nodeValue);n.nodeValue=translateString(originals.get(n));}
  else if(n.nodeType===Node.ELEMENT_NODE)applyNode(n);
 }))).observe(document.body,{childList:true,subtree:true});
}
