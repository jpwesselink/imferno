import React, { useState, useMemo } from "react";

const I = {
  Package: (p: any) => <svg {...{width:16,height:16,viewBox:"0 0 24 24",fill:"none",stroke:"currentColor",strokeWidth:2,strokeLinecap:"round",strokeLinejoin:"round",...p}}><path d="M16.5 9.4 7.55 4.24"/><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.29 7 12 12 20.71 7"/><line x1="12" x2="12" y1="22" y2="12"/></svg>,
  Film: (p: any) => <svg {...{width:14,height:14,viewBox:"0 0 24 24",fill:"none",stroke:"currentColor",strokeWidth:2,strokeLinecap:"round",strokeLinejoin:"round",...p}}><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M7 3v18"/><path d="M3 7.5h4"/><path d="M3 12h18"/><path d="M3 16.5h4"/><path d="M17 3v18"/><path d="M17 7.5h4"/><path d="M17 16.5h4"/></svg>,
  Music: (p: any) => <svg {...{width:14,height:14,viewBox:"0 0 24 24",fill:"none",stroke:"currentColor",strokeWidth:2,strokeLinecap:"round",strokeLinejoin:"round",...p}}><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>,
  Check: (p: any) => <svg {...{width:14,height:14,viewBox:"0 0 24 24",fill:"none",stroke:"currentColor",strokeWidth:2,strokeLinecap:"round",strokeLinejoin:"round",...p}}><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><path d="m9 11 3 3L22 4"/></svg>,
  AlertTriangle: (p: any) => <svg {...{width:14,height:14,viewBox:"0 0 24 24",fill:"none",stroke:"currentColor",strokeWidth:2,strokeLinecap:"round",strokeLinejoin:"round",...p}}><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>,
  ChevronDown: (p: any) => <svg {...{width:14,height:14,viewBox:"0 0 24 24",fill:"none",stroke:"currentColor",strokeWidth:2,strokeLinecap:"round",strokeLinejoin:"round",...p}}><path d="m6 9 6 6 6-6"/></svg>,
  Layers: (p: any) => <svg {...{width:14,height:14,viewBox:"0 0 24 24",fill:"none",stroke:"currentColor",strokeWidth:2,strokeLinecap:"round",strokeLinejoin:"round",...p}}><path d="m12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83Z"/><path d="m22.54 12.43-10 4.55a2 2 0 0 1-1.66 0l-9.4-4.28"/><path d="m22.54 16.43-10 4.55a2 2 0 0 1-1.66 0l-9.4-4.28"/></svg>,
  Globe: (p: any) => <svg {...{width:12,height:12,viewBox:"0 0 24 24",fill:"none",stroke:"currentColor",strokeWidth:2,strokeLinecap:"round",strokeLinejoin:"round",...p}}><circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/><path d="M2 12h20"/></svg>,
  Clock: (p: any) => <svg {...{width:12,height:12,viewBox:"0 0 24 24",fill:"none",stroke:"currentColor",strokeWidth:2,strokeLinecap:"round",strokeLinejoin:"round",...p}}><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>,
  Monitor: (p: any) => <svg {...{width:12,height:12,viewBox:"0 0 24 24",fill:"none",stroke:"currentColor",strokeWidth:2,strokeLinecap:"round",strokeLinejoin:"round",...p}}><rect width="20" height="14" x="2" y="3" rx="2"/><line x1="8" x2="16" y1="21" y2="21"/><line x1="12" x2="12" y1="17" y2="21"/></svg>,
  Marker: (p: any) => <svg {...{width:14,height:14,viewBox:"0 0 24 24",fill:"none",stroke:"currentColor",strokeWidth:2,strokeLinecap:"round",strokeLinejoin:"round",...p}}><path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z"/><circle cx="12" cy="10" r="3"/></svg>,
};

const seqTypeLabel = (t: string) => ({MainImageSequence:"Video",MainAudioSequence:"Audio",SubtitlesSequence:"Subtitles",HearingImpairedCaptionsSequence:"HI Captions",ForcedNarrativeSequence:"Forced Narrative",IABSequence:"IAB Audio"} as any)[t]||t;
const seqTypeSortOrder: Record<string, number> = {MainImageSequence:0,MainAudioSequence:1,IABSequence:2,SubtitlesSequence:3,ForcedNarrativeSequence:4,HearingImpairedCaptionsSequence:5};
const seqTypeColor = (type: string) => ({MainImageSequence:{fill:"#3b82f6",fillBg:"rgba(59,130,246,0.12)"},MainAudioSequence:{fill:"#a855f7",fillBg:"rgba(168,85,247,0.1)"},SubtitlesSequence:{fill:"#22c55e",fillBg:"rgba(34,197,94,0.1)"},ForcedNarrativeSequence:{fill:"#f97316",fillBg:"rgba(249,115,22,0.1)"},HearingImpairedCaptionsSequence:{fill:"#eab308",fillBg:"rgba(234,179,8,0.1)"},IABSequence:{fill:"#ec4899",fillBg:"rgba(236,72,153,0.1)"}} as any)[type]||{fill:"#94a3b8",fillBg:"rgba(148,163,184,0.1)"};
const audioTypeLabel = (t: string) => ({DOLBY_ATMOS:"Dolby Atmos",DOLBY_DIGITAL_PLUS:"DD+",DOLBY_DIGITAL:"DD",STEREO:"Stereo"} as any)[t]||t;
const contentKindLabel = (t: string) => ({PRM:"Primary",VI:"Visually Impaired",HI:"Hearing Impaired",CM:"Commentary"} as any)[t]||t;
const dynRangeLabel = (t: string) => ({SDR:"SDR",HDR10:"HDR10",HDR_DOLBY_VISION:"Dolby Vision",HLG:"HLG"} as any)[t]||t;
const truncUuid = (u: string) => u ? u.substring(0,8)+"\u2026" : "\u2014";
const CopyUuid = ({value}: {value: string | null | undefined}) => {
  if(!value) return <span>{"\u2014"}</span>;
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard.writeText(value).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    }).catch(()=>{});
  };
  return (
    <span onClick={copy} className="relative cursor-pointer hover:text-orange-400 transition-colors group">
      {truncUuid(value)}
      <span className="pointer-events-none absolute left-1/2 -translate-x-1/2 -top-8 px-2 py-1 rounded text-[10px] font-medium whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity bg-zinc-800 text-zinc-100 shadow-lg z-50">
        {copied ? "\u2705 Copied!" : value}
      </span>
    </span>
  );
};
const framesToTC = (frames: number | null | undefined, er: string) => { if(frames==null||!er)return"\u2014"; const p=er.trim().split(/\s+/),n=+p[0],d=p[1]?+p[1]:1; if(!n||!d)return String(frames); const fps=n/d,ts=frames/fps; return `${String(Math.floor(ts/3600)).padStart(2,"0")}:${String(Math.floor((ts%3600)/60)).padStart(2,"0")}:${String(Math.floor(ts%60)).padStart(2,"0")}:${String(Math.round((ts-Math.floor(ts))*fps)).padStart(2,"0")}`; };
const samplesToTC = (samples: number | null | undefined, er: string) => { if(samples==null||!er)return"\u2014"; const p=er.trim().split(/\s+/),n=+p[0],d=p[1]?+p[1]:1; if(!n||!d)return String(samples); const ts=samples/(n/d); return `${String(Math.floor(ts/3600)).padStart(2,"0")}:${String(Math.floor((ts%3600)/60)).padStart(2,"0")}:${String(Math.floor(ts%60)).padStart(2,"0")}.${String(Math.round((ts-Math.floor(ts))*1000)).padStart(3,"0")}`; };
const durationToTC = (count: number | null | undefined, er: string) => { if(count==null||!er)return"\u2014"; return +er.trim().split(/\s+/)[0]>=8000?samplesToTC(count,er):framesToTC(count,er); };
const toSeconds = (count: number | null | undefined, er: string) => { if(count==null||!er)return 0; const p=er.trim().split(/\s+/),n=+p[0],d=p[1]?+p[1]:1; return(!n||!d)?0:count/(n/d); };

const bv: Record<string, string> = {default:"bg-zinc-100 text-zinc-500",blue:"bg-blue-500/10 text-blue-500 border border-blue-500/20",purple:"bg-purple-500/10 text-purple-500 border border-purple-500/20",green:"bg-green-500/10 text-green-500 border border-green-500/20",amber:"bg-yellow-500/10 text-yellow-600 border border-yellow-500/20",red:"bg-red-500/10 text-red-500 border border-red-500/20",pink:"bg-pink-500/10 text-pink-500 border border-pink-500/20",outline:"bg-transparent text-zinc-600 border border-zinc-200"};
const Badge = ({children,variant="default",className=""}: {children: React.ReactNode; variant?: string; className?: string}) => <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[11px] font-medium leading-[18px] whitespace-nowrap ${bv[variant]||bv.default} ${className}`}>{children}</span>;
const Th = ({children}: {children: React.ReactNode}) => <th className="px-3 py-2 text-left text-[10px] font-semibold text-zinc-400 uppercase tracking-wider border-b border-zinc-200">{children}</th>;

const SequenceTimeline = ({sequences,maxDuration,tracks,editRate}: any) => {
  const [expanded,setExpanded] = useState<Set<any>>(new Set());
  if(!sequences?.length) return null;
  const seqMeta = useMemo(() => {
    const lang: any={},mca: any={},kind: any={};
    for(const t of [...(tracks?.AUDIO||[]),...(tracks?.SUBTITLES||[]),...(tracks?.CAPTIONS||[]),...(tracks?.FORCED_NARRATIVE||[])]) if(t.sequenceNumber!=null&&t.language&&!lang[t.sequenceNumber]) lang[t.sequenceNumber]=t.language.toUpperCase();
    for(const t of(tracks?.AUDIO||[])) if(t.sequenceNumber!=null){ if(t.mcaTagName&&!mca[t.sequenceNumber])mca[t.sequenceNumber]=t.mcaTagName; if(t.audioContentKind&&!kind[t.sequenceNumber])kind[t.sequenceNumber]=t.audioContentKind; }
    return{lang,mca,kind};
  },[tracks]);
  const buildLabel = (seq: any) => {
    const p=[seqTypeLabel(seq.type)];
    const l=seqMeta.lang[seq.sequenceNumber]||(seq.language?seq.language.toUpperCase():null);
    if(l)p.push(l);
    // Soundfield / channel info (e.g. "5.1", "Atmos", "2ch")
    const sf=seq.soundfield||seqMeta.mca[seq.sequenceNumber];
    if(sf)p.push(sf);
    else if(seq.channelCount)p.push(seq.channelCount+"ch");
    const k=seqMeta.kind[seq.sequenceNumber];
    if(k&&k!=="PRM")p.push(contentKindLabel(k));
    return p.join(" \u00b7 ");
  };
  const toggle = (id: any) => setExpanded(p=>{const n=new Set(p);n.has(id)?n.delete(id):n.add(id);return n;});
  const resourceRows = (seq: any) => { let o=0; return seq.sequenceResources.map((r: any)=>{const row={...r,_offset:o};o+=r.sourceDuration||0;return row;}); };

  const sorted = useMemo(() => [...sequences].sort((a: any, b: any) =>
    (seqTypeSortOrder[a.type] ?? 99) - (seqTypeSortOrder[b.type] ?? 99)
  ), [sequences]);

  return (
    <div className="flex flex-col gap-0.5">
      {sorted.map((seq: any,i: number) => {
        const c=seqTypeColor(seq.type), totalSec=seq.sequenceResources.reduce((s: number,r: any)=>s+toSeconds(r.sourceDuration,r.editRate||editRate),0);
        const fillPct=maxDuration>0?Math.max((totalSec/maxDuration)*100,2):100, isOpen=expanded.has(seq.id||i), seqId=seq.id||i;
        return (
          <div key={seqId}>
            <div onClick={()=>toggle(seqId)} className="flex items-center gap-2 cursor-pointer rounded py-0.5 hover:bg-zinc-50 transition-colors">
              <div className="w-48 flex-shrink-0 flex items-center gap-1.5 pl-1">
                <span className={`flex transition-transform duration-200 ${isOpen?"":"-rotate-90"}`} style={{color:c.fill}}><I.ChevronDown width={12} height={12}/></span>
                <span className="text-[11px] font-semibold truncate" style={{color:c.fill}}>{buildLabel(seq)}</span>
              </div>
              <div className="flex-1 h-5 bg-zinc-100 rounded overflow-hidden border border-zinc-200/50 relative">
                <div className="h-full rounded relative flex" style={{width:`${fillPct}%`}}>
                  {seq.sequenceResources.map((res: any,ri: number)=>{const td=seq.sequenceResources.reduce((s: number,r: any)=>s+toSeconds(r.sourceDuration,r.editRate||editRate),0),rs=toSeconds(res.sourceDuration,res.editRate||editRate),pct=td>0?(rs/td)*100:100;
                    return <div key={ri} className="h-full relative flex items-center" style={{width:`${Math.max(pct,6)}%`,minWidth:"20px",background:c.fillBg,borderLeft:ri===0?`2.5px solid ${c.fill}`:`1px solid ${c.fill}40`}}>
                      <span className="text-[9px] font-semibold px-1.5 truncate" style={{color:c.fill}}>{`R${ri+1}`}</span>
                    </div>;
                  })}
                </div>
              </div>
              <div className="w-12 flex-shrink-0 text-right pr-2 text-[10px] font-mono text-zinc-400">Seq {seq.sequenceNumber??"\u2014"}</div>
            </div>
            {isOpen && (
              <div className="ml-[198px] mr-[60px] mt-0.5 mb-1.5 border border-zinc-200 rounded-md overflow-hidden text-[11px]">
                <table className="w-full border-collapse"><thead><tr className="bg-zinc-50">
                  {["#","Offset","Track File","Duration","Entry Point","Encoding"].map(h=><th key={h} className="px-2.5 py-1 text-left text-[10px] font-semibold text-zinc-400 uppercase tracking-wider border-b border-zinc-200">{h}</th>)}
                </tr></thead><tbody>
                  {resourceRows(seq).map((res: any,ri: number)=>{const rr=res.editRate||editRate; return(
                    <React.Fragment key={res.id||ri}>
                      <tr className="border-b border-zinc-100 last:border-0 hover:bg-zinc-50/50">
                        <td className="px-2.5 py-1 font-mono text-zinc-400"><span className="px-1.5 py-px rounded text-[10px] font-semibold" style={{background:c.fillBg,color:c.fill}}>R{ri+1}</span></td>
                        <td className="px-2.5 py-1 font-mono">{durationToTC(res._offset,rr)}</td>
                        <td className="px-2.5 py-1 font-mono text-zinc-400"><CopyUuid value={res.trackFileId}/></td>
                        <td className="px-2.5 py-1 font-mono">{durationToTC(res.sourceDuration,rr)}{res.intrinsicDuration!=null&&res.sourceDuration!==res.intrinsicDuration&&<span className="text-zinc-400 text-[10px]"> / {durationToTC(res.intrinsicDuration,rr)}</span>}</td>
                        <td className="px-2.5 py-1 font-mono text-zinc-400">{res.entryPoint!=null?durationToTC(res.entryPoint,rr):"\u2014"}</td>
                        <td className="px-2.5 py-1 font-mono text-[10px] text-zinc-400"><CopyUuid value={res.sourceEncoding}/></td>
                      </tr>
                    </React.Fragment>);})}
                </tbody></table>
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
};

const TrackTable = ({tracks}: any) => {
  const all = useMemo(()=>{const r: any[]=[];["VIDEO","AUDIO","SUBTITLES","CAPTIONS","FORCED_NARRATIVE"].forEach(c=>(tracks[c]||[]).forEach((t: any)=>r.push({...t,_cat:c})));return r;},[tracks]);
  if(!all.length)return null;
  const cc: any={VIDEO:"blue",AUDIO:"purple",SUBTITLES:"green",CAPTIONS:"amber",FORCED_NARRATIVE:"amber"};
  return (
    <div className="border border-zinc-200 rounded-lg overflow-hidden text-xs">
      <table className="w-full border-collapse">
        <thead><tr className="bg-zinc-50">{["#","Type","Codec / Format","Details","Language","Fragment","Track ID","Seq Track"].map(h=><Th key={h}>{h}</Th>)}</tr></thead>
        <tbody>{all.map((t: any,i: number)=>(
          <tr key={`${t._cat}-${t.trackNumber}-${i}`} className="border-b border-zinc-100 last:border-0 hover:bg-zinc-50 transition-colors">
            <td className="px-3 py-1.5 font-mono text-zinc-400">{t.trackNumber}</td>
            <td className="px-3 py-1.5"><Badge variant={cc[t._cat]||"default"}>{t._cat==="FORCED_NARRATIVE"?"FN":t._cat}</Badge></td>
            <td className="px-3 py-1.5">{t._cat==="VIDEO"?<span className="flex items-center gap-1.5"><span className="font-medium">{t.width}\u00d7{t.height}</span><Badge variant="outline">{t.quality}</Badge><Badge variant={t.dynamicRange?.includes("HDR")||t.dynamicRange?.includes("DOLBY")?"pink":"outline"}>{dynRangeLabel(t.dynamicRange)}</Badge></span>:t._cat==="AUDIO"?<span className="flex items-center gap-1.5"><span className="font-medium">{audioTypeLabel(t.type)}</span>{t.mcaTagName?<Badge variant="outline">{t.mcaTagName}</Badge>:t.channelCount>0?<Badge variant="outline">{t.channelCount===6?"5.1":t.channelCount===8?"7.1":`${t.channelCount}ch`}</Badge>:null}{t.atmosType&&<Badge variant="purple">{t.atmosType}</Badge>}</span>:<span className="font-medium">TTML</span>}</td>
            <td className="px-3 py-1.5 text-zinc-500">{t._cat==="AUDIO"?contentKindLabel(t.audioContentKind):"\u2014"}</td>
            <td className="px-3 py-1.5">{t.language?<span className="inline-flex items-center gap-1"><I.Globe/>{t.language.toUpperCase()}</span>:"\u2014"}</td>
            <td className="px-3 py-1.5 font-mono text-[11px] text-zinc-500">{t.fragmentDuration||"\u2014"}</td>
            <td className="px-3 py-1.5 font-mono text-[11px] text-zinc-400"><CopyUuid value={t.trackIdentifier}/></td>
            <td className="px-3 py-1.5 font-mono text-[11px] text-zinc-400"><CopyUuid value={t.sequenceTrackId}/></td>
          </tr>))}</tbody>
      </table>
    </div>
  );
};

const CplCard = ({cpl,isOpen,onToggle}: any) => {
  const sa=cpl.sourceAsset||{}, [activeTab,setActiveTab]=useState("timeline");
  const maxDuration = useMemo(()=>{ if(!sa.sequences)return 0; return Math.max(...sa.sequences.map((s: any)=>s.sequenceResources.reduce((sum: number,r: any)=>sum+toSeconds(r.sourceDuration,r.editRate||sa.editRate),0))); },[sa.sequences,sa.editRate]);
  const kc: any={FEATURE:{badge:"blue",icon:"\ud83c\udfac"},TRAILER:{badge:"purple",icon:"\ud83c\udf9e\ufe0f"},SHORT:{badge:"green",icon:"\ud83d\udcfd\ufe0f"}}, k=kc[sa.contentKind]||{badge:"default",icon:"\ud83d\udce6"};
  const hasTrackData = Object.values(sa.tracks||{}).some((arr: any)=>arr?.length>0);
  const tabs=[{id:"timeline",label:"Timeline",icon:<I.Layers/>},...(hasTrackData?[{id:"tracks",label:"Tracks",icon:<I.Music/>}]:[]),{id:"markers",label:`Markers${cpl.markers?.length?` (${cpl.markers.length})`:""}`,icon:<I.Marker/>},{id:"metadata",label:"Metadata",icon:<I.Package/>}];
  const segIds=[...new Set((sa.sequences||[]).map((s: any)=>s.segmentId).filter(Boolean))];

  return (
    <div className={`border rounded-xl overflow-hidden bg-white transition-all duration-200 ${isOpen?"border-zinc-300 shadow-sm ring-1 ring-zinc-200/50":"border-zinc-200 hover:border-zinc-300"}`}>
      <button onClick={onToggle} className="w-full flex items-center gap-3 px-4 py-3.5 text-left hover:bg-zinc-50 transition-colors">
        <span className={`flex transition-transform duration-200 ${isOpen?"":"-rotate-90"} ${isOpen?"text-amber-600":"text-zinc-400"}`}><I.ChevronDown/></span>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            {sa.contentKind&&<Badge variant={k.badge}>{k.icon} {sa.contentKind}</Badge>}
            {cpl.isSupplemental&&<Badge variant="amber">Supplemental</Badge>}
            {sa.territory&&<Badge variant="outline"><I.Globe/> {sa.territory}</Badge>}
            {sa.duration&&<Badge variant="outline"><I.Clock/> {sa.duration}</Badge>}
            {sa.videoQuality&&<Badge variant="outline"><I.Monitor/> {sa.videoQuality} {dynRangeLabel(sa.videoDynamicRange)}</Badge>}
            {sa.frameRate&&<Badge variant="outline">{sa.frameRate} fps</Badge>}
            {sa.audioType&&sa.audioType!=="STEREO"&&<Badge variant="purple">{audioTypeLabel(sa.audioType)}</Badge>}
            {cpl.applicationProfile&&<Badge variant="outline">{cpl.applicationProfile}</Badge>}
            <Badge variant="outline"><I.Layers/> {cpl.segmentCount??0} segment{(cpl.segmentCount??0)!==1?"s":""}</Badge>
          </div>
          <div className="mt-1.5 text-xs font-mono text-zinc-400 truncate">{cpl.title}</div>
        </div>
        <div className="flex gap-1 flex-shrink-0 items-center text-[11px] text-zinc-400">
          <span className="flex items-center gap-0.5"><I.Layers/> {sa.sequences?.length||0} seq</span>
        </div>
      </button>
      {isOpen&&(
        <div className="border-t border-zinc-200">
          <div className="flex gap-1 bg-zinc-100 rounded-lg p-1 mx-4 mt-3 mb-1 w-fit">
            {tabs.map(tab=><button key={tab.id} onClick={()=>setActiveTab(tab.id)} className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all ${activeTab===tab.id?"bg-white text-zinc-900 shadow-sm":"text-zinc-500 hover:text-zinc-700"}`}>{tab.icon} {tab.label}</button>)}
          </div>
          <div className="p-4">
            {activeTab==="timeline"&&<SequenceTimeline sequences={sa.sequences} maxDuration={maxDuration} tracks={sa.tracks} editRate={sa.editRate}/>}
            {activeTab==="tracks"&&sa.tracks&&<TrackTable tracks={sa.tracks}/>}
            {activeTab==="markers"&&(cpl.markers?.length>0?(
              <div className="border border-zinc-200 rounded-lg overflow-hidden text-xs"><table className="w-full border-collapse"><thead><tr className="bg-zinc-50">{["Label","Offset","Scope"].map(h=><Th key={h}>{h}</Th>)}</tr></thead><tbody>
                {cpl.markers.map((m: any,i: number)=><tr key={i} className="border-b border-zinc-100 last:border-0 hover:bg-zinc-50"><td className="px-3 py-1.5 font-medium">{m.label||"\u2014"}</td><td className="px-3 py-1.5 font-mono text-zinc-500">{m.offset??"\u2014"}</td><td className="px-3 py-1.5 text-zinc-500">{m.scope||"\u2014"}</td></tr>)}
              </tbody></table></div>
            ):<p className="text-xs text-zinc-400 text-center py-8">No markers defined in this CPL.</p>)}
            {activeTab==="metadata"&&(
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                {[{l:"CPL UUID",v:(cpl.id||"").replace("urn:uuid:",""),m:true},...(sa.contentTitle?[{l:"Content Title",v:sa.contentTitle}]:[]),{l:"Edit Rate",v:sa.editRate||"\u2014"},{l:"Frame Rate",v:sa.frameRate?`${sa.frameRate} fps`:"\u2014"},{l:"Type",v:cpl.isSupplemental?"Supplemental":"Original Version"},...(cpl.applicationProfile?[{l:"Application Profile",v:cpl.applicationProfile}]:[]),...(cpl.segmentCount!=null?[{l:"Segments",v:cpl.segmentCount}]:[]),...(segIds.length>0?[{l:"Segment ID",v:segIds.map((id: string)=>id.substring(0,8)+"\u2026").join(", "),m:true}]:[]),...(cpl.timecodeStart?[{l:"Timecode Start",v:cpl.timecodeStart,m:true}]:[]),{l:"Audio Languages",v:sa.audioLanguages?.map((l: string)=>l.toUpperCase()).join(", ")||"\u2014"},{l:"Subtitle Languages",v:sa.subtitleLanguages?.map((l: string)=>l.toUpperCase()).join(", ")||"\u2014"},{l:"Forced Narrative",v:sa.forcedNarrativeLanguages?.map((l: string)=>l.toUpperCase()).join(", ")||"\u2014"}].map((item: any)=>(
                  <div key={item.l} className="px-3 py-2.5 rounded-lg bg-zinc-50 border border-zinc-100">
                    <div className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 mb-1">{item.l}</div>
                    <div className={`text-xs font-medium text-zinc-900 break-all ${item.m?"font-mono":""}`}>{item.v}</div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

export default function IMFPackageViewer({data}: {data: any}) {
  const [openCpls,setOpenCpls]=useState<Set<string>>(()=>new Set([data?.cpls?.[0]?.id].filter(Boolean)));
  const [showValidation,setShowValidation]=useState(false);
  const dataId=data?.package?.assetMapId;
  const [lastDataId,setLastDataId]=useState(dataId);
  if(dataId&&dataId!==lastDataId){setLastDataId(dataId);setOpenCpls(new Set([data.cpls?.[0]?.id].filter(Boolean)));}
  const toggleCpl=(id: string)=>{setOpenCpls(p=>{const n=new Set(p);n.has(id)?n.delete(id):n.add(id);return n;});};
  const expandAll=()=>setOpenCpls(new Set(data?.cpls?.map((c: any)=>c.id)||[]));
  const collapseAll=()=>setOpenCpls(new Set());
  if(!data?.package) return <div className="flex flex-col items-center justify-center py-16 px-6 gap-3 text-zinc-400"><I.Package/><p className="text-sm font-medium">No package loaded</p></div>;
  const v=data.validation;
  return (
    <div className="w-full bg-white rounded-2xl px-6 py-6 text-zinc-900" style={{fontFamily:"-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif"}}>
      <div className="flex items-start justify-between mb-6 gap-4 flex-wrap">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-lg bg-amber-600 text-white flex items-center justify-center"><I.Package/></div>
          <div><h1 className="text-lg font-bold leading-tight">IMF Package</h1><p className="text-xs text-zinc-400 font-mono">{(data.package.assetMapId||"").replace("urn:uuid:","")}</p></div>
        </div>
        {v&&<button onClick={()=>setShowValidation(s=>!s)} className={`flex items-center gap-2 px-3.5 py-2 rounded-lg border font-semibold text-[13px] leading-none transition-all cursor-pointer ${v.valid?"bg-green-500/5 text-green-600 border-green-500/25 hover:bg-green-500/10":"bg-red-500/5 text-red-600 border-red-500/25 hover:bg-red-500/10"}`}>
          {v.valid?<I.Check/>:<I.AlertTriangle/>}{v.valid?"Valid":`${v.issues?.length||0} Issues`}<span className={`flex opacity-60 transition-transform duration-200 ${showValidation?"":"-rotate-90"}`}><I.ChevronDown/></span>
        </button>}
      </div>
      {showValidation&&v&&(
        <div className={`mb-4 px-4 py-3.5 rounded-lg border ${v.valid?"bg-green-50/50 border-green-500/15":"bg-red-50/50 border-red-500/15"}`}>
          <div className={`text-[11px] font-semibold uppercase tracking-wider mb-2 ${v.valid?"text-green-600":"text-red-600"}`}>Validated by Imferno</div>
          {v.valid?<p className="text-xs text-zinc-500 leading-relaxed">Package structure validated successfully.</p>:(
            <div className="flex flex-col gap-1.5">
              {(v.issues||[]).map((issue: any,i: number)=>{
                const sev = (issue.severity||"").toLowerCase();
                const sevColor = sev === "critical" || sev === "error" ? "text-red-600" : sev === "warning" ? "text-amber-600" : "text-zinc-400";
                const sevLabel = sev === "critical" ? "critical" : sev === "error" ? "error" : sev === "warning" ? "warning" : "info";
                const SevIcon = sev === "critical" || sev === "error" ? I.AlertTriangle : sev === "warning" ? I.AlertTriangle : I.Check;
                const cplShort = issue.cplId ? issue.cplId.substring(0, 8) : null;
                return <div key={i} className="flex items-start gap-2 text-xs leading-relaxed">
                  <span className={`flex-shrink-0 mt-0.5 opacity-60 ${sevColor}`}><SevIcon/></span>
                  <div>
                    <span className={`font-semibold ${sevColor}`}>{sevLabel}</span>
                    {issue.code && <span className="font-mono text-zinc-500 ml-1.5">{issue.code}</span>}
                    {cplShort && <span className="text-zinc-400 ml-1.5">[CPL:{cplShort}]</span>}
                    <div className="text-zinc-600 mt-0.5">{typeof issue==="string"?issue:issue.message||JSON.stringify(issue)}</div>
                  </div>
                </div>;
              })}
            </div>
          )}
        </div>
      )}
      <div className="grid grid-cols-3 gap-2 mb-5">
        {[{l:"CPLs",v:data.package.cplCount},{l:"Assets",v:data.package.assetCount},{l:"PKLs",v:data.package.pklCount}].map(s=>(
          <div key={s.l} className="px-3 py-2.5 rounded-lg border border-zinc-200 bg-white">
            <div className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 mb-0.5">{s.l}</div>
            <div className="text-sm font-semibold">{s.v}</div>
          </div>
        ))}
      </div>
      <div className="flex items-center justify-between mb-2.5">
        <h2 className="text-sm font-semibold flex items-center gap-1.5"><span className="text-amber-600"><I.Layers/></span> Composition Playlists <span className="font-normal text-zinc-400">({data.cpls?.length||0})</span></h2>
        <div className="flex gap-1">
          <button onClick={expandAll} className="px-2.5 py-1 text-[11px] font-medium border border-zinc-200 rounded-md bg-white text-zinc-400 hover:bg-zinc-50 hover:text-amber-600 cursor-pointer transition-colors">Expand All</button>
          <button onClick={collapseAll} className="px-2.5 py-1 text-[11px] font-medium border border-zinc-200 rounded-md bg-white text-zinc-400 hover:bg-zinc-50 hover:text-amber-600 cursor-pointer transition-colors">Collapse</button>
        </div>
      </div>
      <div className="flex flex-col gap-3">{(data.cpls||[]).map((cpl: any)=><CplCard key={cpl.id} cpl={cpl} isOpen={openCpls.has(cpl.id)} onToggle={()=>toggleCpl(cpl.id)}/>)}</div>
    </div>
  );
}
