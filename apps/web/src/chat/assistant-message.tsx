import type { RenderDocument } from '../runtime-types';

export function AssistantMessage({ render }: { render: RenderDocument }) {
  return (
    <div className="msg-body">
      {render.blocks.map((block, index) => {
        switch (block.type) {
          case 'heading':
            return (
              <div className="msg-heading" key={`heading-${index}`}>
                {block.text}
              </div>
            );
          case 'paragraph':
            return (
              <div className="msg-paragraph" key={`paragraph-${index}`}>
                {block.text}
              </div>
            );
          case 'bullet_list':
            return (
              <ul className="msg-bullet-list" key={`list-${index}`}>
                {block.items.map((item, itemIndex) => (
                  <li key={`item-${index}-${itemIndex}`}>{item}</li>
                ))}
              </ul>
            );
          case 'code_block':
            return (
              <pre className="msg-code-block" key={`code-${index}`}>
                <code>{block.code}</code>
              </pre>
            );
          case 'citations':
            return (
              <div className="msg-citations" key={`citations-${index}`}>
                Sources: {block.sources.join(', ')}
              </div>
            );
          default:
            return null;
        }
      })}
    </div>
  );
}
