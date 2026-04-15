import Link from '@docusaurus/Link';

import {magneticFieldEvents} from '@site/src/components/magneticField';

import styles from './styles.module.css';

export type TermRampItem = {
  plain: string;
  term: string;
  body: string;
  href: string;
  plainLabel?: string;
  termLabel?: string;
};

type TermRampProps = {
  items: TermRampItem[];
  tone?: 'docs' | 'hero';
};

export default function TermRamp({
  items,
  tone = 'docs',
}: TermRampProps) {
  return (
    <div
      className={`${styles.grid} ${tone === 'hero' ? styles.hero : styles.docs}`}>
      {items.map((item) => (
        <Link
          key={item.term}
          className={styles.cardLink}
          to={item.href}
          {...(tone === 'hero' ? magneticFieldEvents<HTMLAnchorElement>() : {})}>
          <article className={styles.card}>
            <div className={styles.plainBlock}>
              <p className={styles.plainLabel}>
                {item.plainLabel ?? 'Everyday language'}
              </p>
              <p className={styles.plain}>{item.plain}</p>
            </div>
            <div className={styles.arrow} aria-hidden="true" />
            <div className={styles.keelBlock}>
              <p className={styles.keelLabel}>{item.termLabel ?? 'Paddles term'}</p>
              <h3>{item.term}</h3>
              <p>{item.body}</p>
            </div>
          </article>
        </Link>
      ))}
    </div>
  );
}
