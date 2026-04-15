import Link from '@docusaurus/Link';

import {magneticFieldEvents} from '@site/src/components/magneticField';

import styles from './styles.module.css';

export type SignalItem = {
  eyebrow?: string;
  title: string;
  body: string;
  href?: string;
  cta?: string;
};

type SignalGridProps = {
  items: SignalItem[];
  columns?: 'two' | 'three';
  tone?: 'docs' | 'hero';
};

export default function SignalGrid({
  items,
  columns = 'three',
  tone = 'docs',
}: SignalGridProps) {
  return (
    <div
      className={`${styles.grid} ${
        columns === 'two' ? styles.twoColumns : styles.threeColumns
      } ${tone === 'hero' ? styles.hero : styles.docs}`}>
      {items.map((item) => {
        const magneticProps =
          tone === 'hero' ? magneticFieldEvents<HTMLAnchorElement>() : {};
        const content = (
          <>
            {item.eyebrow ? (
              <p className={styles.eyebrow}>{item.eyebrow}</p>
            ) : null}
            <h3>{item.title}</h3>
            <p>{item.body}</p>
            {item.href ? (
              <span className={styles.linkText}>{item.cta ?? 'Read more'}</span>
            ) : null}
          </>
        );

        if (item.href) {
          return (
            <Link
              key={item.title}
              className={styles.cardLink}
              to={item.href}
              {...magneticProps}>
              <article className={styles.card}>{content}</article>
            </Link>
          );
        }

        return (
          <article key={item.title} className={styles.card}>
            {content}
          </article>
        );
      })}
    </div>
  );
}
