import type { CustomHtmlBlock as CustomHtmlBlockType } from "@/types";

export default function CustomHtmlBlock({
  block,
}: {
  block: CustomHtmlBlockType;
}) {
  return (
    <iframe
      title={`custom-html-${block.id}`}
      srcDoc={block.html}
      sandbox=""
      referrerPolicy="no-referrer"
      className="my-4 h-80 w-full overflow-hidden rounded-lg border border-gray-200 bg-white"
    />
  );
}
