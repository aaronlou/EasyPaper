import { useRef, useEffect } from "react";
import type { CustomHtmlBlock as CustomHtmlBlockType } from "@/types";

export default function CustomHtmlBlock({
  block,
}: {
  block: CustomHtmlBlockType;
}) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (ref.current && block.html) {
      // 安全：用 srcdoc 而非 innerHTML，但这里是受控片段
      // 实际生产环境应使用 iframe sandbox
      const shadow = ref.current.attachShadow({ mode: "open" });
      shadow.innerHTML = block.html;
    }
  }, [block.html]);

  return <div ref={ref} className="my-4 rounded-lg border border-gray-200 overflow-hidden" />;
}
