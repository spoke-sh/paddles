import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    'intro',
    {
      type: 'category',
      label: 'Start Here',
      items: ['start-here/installation', 'start-here/first-turn'],
    },
    {
      type: 'category',
      label: 'Concepts',
      items: [
        'concepts/turn-loop',
        'concepts/recursive-planning',
        'concepts/search-retrieval',
        'concepts/context-tiers',
        'concepts/context-pressure',
        'concepts/model-routing',
      ],
    },
    {
      type: 'category',
      label: 'Reference',
      items: ['reference/foundational-docs', 'reference/native-transports'],
    },
  ],
};

export default sidebars;
