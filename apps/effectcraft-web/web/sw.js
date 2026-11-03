// ACA Motion Lab scoped offline cache. Build replaces the version and file list.
// Network-first runtime assets prevent a cached old native/theme bundle winning
// after an atomic deployment. Offline fallback contains only this app's assets.
const VERSION = "__EC_VERSION__";
const FILES = __EC_FILES__;
const PREFIX = 'effectcraft-'; // Retain and migrate only this app's legacy cache family.
const CACHE = `${PREFIX}${VERSION}`;
const SCOPE = new URL(self.registration.scope);
const KNOWN = new Set(FILES.map(f => new URL(f, SCOPE).pathname));
self.addEventListener('install', event => {
 event.waitUntil(caches.open(CACHE).then(cache => cache.addAll(FILES.map(f => new Request(new URL(f,SCOPE),{cache:'reload'})))).then(()=>self.skipWaiting()));
});
self.addEventListener('activate', event => {
 event.waitUntil(caches.keys().then(keys => Promise.all(keys.filter(k=>k.startsWith(PREFIX)&&k!==CACHE).map(k=>caches.delete(k)))).then(()=>self.clients.claim()));
});
self.addEventListener('message', event => {
 if(event.data?.type==='aca:cache-version'&&event.ports?.[0])event.ports[0].postMessage({type:'aca:cache-version',version:VERSION,scope:SCOPE.pathname});
});
self.addEventListener('fetch', event => {
 const req=event.request,url=new URL(req.url);
 if(req.method!=='GET'||url.origin!==SCOPE.origin||!url.pathname.startsWith(SCOPE.pathname))return;
 if(url.pathname.endsWith('/effectcraft-config.json'))return;
 if(req.mode!=='navigate'&&!KNOWN.has(url.pathname))return;
 event.respondWith((async()=>{
  const cache=await caches.open(CACHE).catch(()=>null);
  const key=req.mode==='navigate'?new URL('./',SCOPE).href:req;
  try{
   const response=await fetch(req,{cache:'no-cache'});
   if(response.ok&&response.type==='basic'){
    if(cache&&!/\b(?:private|no-store)\b/i.test(response.headers.get('cache-control')||''))void cache.put(key,response.clone()).catch(()=>{});
    return response;
   }
   if(response.status<500)return response;
  }catch{/* Only a network/server failure may use this app's offline copy. */}
  const hit=await cache?.match(key,{ignoreSearch:req.mode==='navigate'}).catch(()=>null);
  return hit||new Response('Motion Lab is unavailable offline. Reconnect and reload; browser recovery was preserved.',{status:503,headers:{'Content-Type':'text/plain; charset=utf-8'}});
 })());
});
