import Link from '@docusaurus/Link';

import {magneticFieldEvents} from '@site/src/components/magneticField';

import styles from './styles.module.css';

export type TurnStep = {
  label: string;
  command: string;
  body: string;
  href: string;
};

type TurnCycleProps = {
  items: TurnStep[];
  tone?: 'docs' | 'hero';
};

export default function TurnCycle({
  items,
  tone = 'docs',
}: TurnCycleProps) {
  return (
    <div
      className={`${styles.wrap} ${tone === 'hero' ? styles.hero : styles.docs}`}>
      {items.map((step, index) => (
        <Link
          key={step.label}
          className={styles.cardLink}
          to={step.href}
          {...(tone === 'hero' ? magneticFieldEvents<HTMLAnchorElement>() : {})}>
          <article className={styles.card}>
            <div className={styles.head}>
              <div className={styles.markerRow}>
                <span className={styles.count}>{`0${index + 1}`}</span>
                <h3>{step.label}</h3>
              </div>
            </div>
            <code>{step.command}</code>
            <p>{step.body}</p>
          </article>
        </Link>
      ))}
    </div>
  );
}
