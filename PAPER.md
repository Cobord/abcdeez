## Adaptive Training for Flexible Skill Acquisition: Building Navigable Mental Models of Graph-Structured Tasks

### 1. Introduction

Many cognitive tasks, from navigating complex software to learning a city's public transport system, involve internalizing **structured knowledge**. While seemingly straightforward, mastering these domains often requires more than rote memorization of a single sequence of steps. For instance, a software user might learn a specific menu path to access a common feature (e.g., `File -> Save As -> Cloud Storage`). This linear, procedural knowledge works well when the environment is static and the goal always accessed in the same way. However, if the menu structure changes, a shortcut is introduced, or the user needs to achieve the goal from a different starting point (e.g., saving from a context menu), this rigid, "one-way street" knowledge breaks down. Users struggle, resort to trial-and-error, or inefficiently retrace familiar paths, even if a shorter, direct route exists. This highlights a fundamental challenge: **mastery of structured knowledge should imply flexible, bidirectional access and adaptive traversal, not just a parroted sequence.**

More generally, many domains are inherently **graph-structured**, representing states (nodes) and relations (edges) such as dependencies, transitions, or ordering rules. Examples include a user interface where screens are nodes and clicks are edges, or a set of assembly instructions that form a partial order where some steps are independent (can be done in any order) while others have prerequisites. Traditional training often inadvertently **linearizes** this rich structure, teaching only one specific total order (e.g., one fixed set of assembly steps, one specific subway route). This can be cognitively adaptive for simple, frequently repeated sequences, as it reduces cognitive load by automating a fixed path. However, in larger, irregular graphs where uniform coverage by rote is infeasible, or where flexibility is critical, such linearization can become a significant bottleneck for transfer and problem-solving. Human studies show that without explicit training, people tend to impose a total order even when presented with partial orders, failing to represent the inherent flexibility. This underscores the need for training that deliberately fosters a **graph-structured mental model** (or a *navigable schema*) and the skills to traverse it flexibly.

We propose an **adaptive training system** designed to build these flexible cognitive skills. Instead of just reinforcing a single sequence, our system helps learners internalize the underlying graph of states and relations. For example, for software navigation, this means understanding all direct links from a given screen, not just the single one used in a tutorial, and knowing alternative paths to a feature. For an alphabet, it means knowing both the successor and predecessor of any letter. Mastery, in our framework, is demonstrated by the ability to fluidly traverse the learned graph in arbitrary ways – for instance, executing a software command via a new shortcut, or listing letters in reverse order. Achieving this requires moving beyond surface-level memorization; it entails building a robust, **graph-coded mental model** capable of supporting diverse operations.

Our core contribution is a **lean, adaptive learning architecture** that fosters these graph-coded mental models. We introduce:
1.  A **unified probabilistic learner model** that estimates the learner's knowledge state over graph structures, accounting for item positions, operation proficiencies, and local confusions.
2.  A **minimal, coverage-optimal task battery** that probes specific invariances defining graph-coded mastery (e.g., bidirectionality, chunk-stitching, topological ordering).
3.  An **adaptive scheduler** that selects tasks to maximize expected information gain about the learner's graph-coded mental model.
4.  **Behavioral metrics** that operationalize graph-coded mastery and predict transfer.
5.  A **pre-registered behavioral study design** to demonstrate improvements on these metrics compared to strong control conditions, using a controlled domain.

### 2. Related Work: Beyond Linear Repetition

Our work builds upon, and contrasts with, several established fields in cognitive science and intelligent tutoring systems:

#### 2.1 Serial Order Memory and Sequence Learning

The literature on human sequence learning (e.g., Hebb, Lashley, Restle, chaining models, positional coding, OSCAR/TDAM) primarily focuses on how people acquire and recall *linear* lists of items. These models often emphasize mechanisms for maintaining order (e.g., associative links between adjacent items, or positional tags). While effective for rote memorization of fixed sequences (like the alphabet song), they typically explain difficulties with non-standard traversals (e.g., backward recall, starting mid-sequence) as a consequence of their unidirectional or fixed-origin encoding. Our approach fundamentally differs by aiming to teach the *bidirectional* and *multi-path* nature of a graph, rather than merely strengthening one fixed linear order. We argue that true mastery involves a flexible mental structure, not just a robust linear chain. This aligns with work on cognitive maps but focuses on the deliberate *training* of abstract relational structures.

#### 2.2 Intelligent Tutoring Systems and Psychometric Models

Intelligent Tutoring Systems (ITS) and psychometric models like Bayesian Knowledge Tracing (BKT) and Item Response Theory (IRT) are foundational for adaptive learning. BKT tracks mastery of binary skills, while IRT models a learner's ability and item difficulty for various types of questions. Our work extends these models significantly:
*   **Multi-dimensionality:** Unlike typical IRT which might model a single ability or a few discrete skills, our model tracks a richer set of parameters related to a graph structure: latent node positions, operation-specific proficiencies (e.g., forward vs. backward traversal, k-jump ability), and specific pairwise confusions.
*   **Graph-Awareness:** Crucially, our model's parameters are inherently linked to the underlying graph structure. For instance, item difficulty in our model is not an arbitrary constant but a function of graph distance, chunk boundaries, or relational complexity. This allows the system to target learning not just on isolated 'skills' but on specific 'links' or 'regions' of the internal graph that are weak.
*   **Structural Uncertainty:** Our model doesn't just estimate what a learner knows, but *where* their mental model deviates from the true graph (e.g., uncertainty in relative ordering, mis-estimated dependencies, or un-stitched chunk boundaries). This allows for targeted intervention.

#### 2.3 Active Learning and Spaced Repetition

Techniques like active recall, spaced repetition (e.g., Leitner system, SM-2 algorithm), and error-driven learning are known to improve long-term retention. These methods primarily optimize for item forgetting curves: revisiting items when they are likely to be forgotten. Our adaptive scheduler incorporates these principles but adds a critical layer of **structure-aware optimization**. Instead of merely optimizing the recall probability of individual *items*, our scheduler prioritizes queries that maximize **expected information gain** about the underlying *graph structure* itself. This includes targeting edges, relational dependencies, specific chunk boundaries, or cycles – areas where the learner's internal graph representation is uncertain or asymmetric. This aims for a more holistic mastery of the *relations* within the domain, not just item recall.

#### 2.4 Cognitive Maps and Spatial Navigation

The concept of a "cognitive map" (Tolman, O'Keefe, Moser) describes how the brain spontaneously forms spatial representations of environments, often leveraging hippocampal mechanisms. While our work deals with abstract, symbolic structures rather than physical space, it draws a parallel: we aim to cultivate a "map-like" representation where elements are connected by learned relations. However, a key distinction is that our system is explicitly designed for *deliberate instruction* and *skill acquisition* of graph traversal policies, rather than passively observing and inferring a map from experience. We proactively guide the learner to build a functional, navigable mental model, which can be queried for planning and pathfinding, akin to using a real map.

### 3. A Probabilistic Model for Graph-Coded Knowledge Acquisition

To effectively guide learning, our system employs a **probabilistic generative model** of the learner's internal state. This model is continuously updated by the learner's responses (accuracy and response time) and, in turn, informs the adaptive selection of new tasks. The goal is to infer a robust, graph-coded mental model for the given domain.

#### 3.1 Domain Representation

A domain is represented as a graph $G=(V, E, R)$, where $V$ is a set of nodes (e.g., software screens, alphabet letters, subway stations), $E$ is a set of directed edges representing possible transitions or ordering relations, and $R$ is a set of additional semantic relations or attributes (e.g., "is_vowel", "is_prerequisite_for").

#### 3.2 Learner's Latent State

The learner's internal knowledge of $G$ is modeled by a set of latent parameters:

*   **Latent Node Embeddings ($z_v$):** For ordered sequences (linear, cyclic), each node $v \in V$ is assigned a latent position or angle.
    *   **Linear Order:** $z_v \in \mathbb{R}$, where $z_u < z_v$ implies $u$ comes before $v$.
    *   **Cyclic Order:** $z_v \in [0, 2\pi)$, representing angular positions on a circle.
    *   **Partial Orders/DAGs:** For $u \to v$, we infer a soft inequality $score(u) < score(v)$, potentially using $z_v \in \mathbb{R}^D$ and enforcing constraints via priors or specific architectures (e.g., hyperbolic embeddings for tree-like structures, though we focus on simpler structures for initial validation).
*   **Operation Proficiencies ($\theta_o$):** Each distinct operation type $o$ (e.g., `successor`, `predecessor`, `k-jump`, `shortest_path_to`) has a proficiency parameter $\theta_o \in \mathbb{R}$, which is a latent ability score akin to an IRT parameter.
*   **Memory Strengths ($s_v(t)$):** Each node $v$ and operation $o$ can have a memory strength $s_v(t)$ that decays over time, modeling forgetting. This allows for principled spaced repetition.
*   **Confusability Kernel ($K_{ij}$):** This non-negative matrix (or kernel function) quantifies the inherent similarity or proneness to confusion between nodes $i$ and $j$. It is parameterized as a decaying function of true graph distance $d_G(i,j)$ (e.g., $w_1 \exp(-\|z_i - z_j\|^2/\rho^2)$), capturing the "locality of errors." It may also include contributions from phonological or visual features if applicable. This allows the model to capture, for example, a higher likelihood of swapping adjacent items, or items with similar sounds.
*   **Chunk Boundaries ($B$):** For sequences, we can infer latent "chunk boundaries" $b \in V$ where the learner implicitly segments the sequence. Traversing across a boundary incurs an additional cognitive cost (increased RT, higher error probability). This can be modeled by an indicator variable $\mathbf{1}[\text{boundary for }q]$ that impacts response likelihoods.

All parameters are typically estimated hierarchically (e.g., partial pooling across users) within a Bayesian framework (e.g., using Stan or variational inference) to leverage population information while accounting for individual differences. Identifiability concerns (e.g., gauge fixing for embeddings, label switching) are handled with appropriate priors or constraints.

#### 3.3 Response Model

For a given query $q=(u,v,op)$ (e.g., "Is $u$ before $v$?", "What is next after $u$?") and a learner's latent state, we model the probability of a correct response $p(y=1|q, z, \theta, s)$ and the response time $RT$.

*   **Accuracy Model (Probabilistic Ranking/Classification):**
    *   **Pairwise Order:** For queries like "Is $u$ before $v$?", we use a Luce/Bradley-Terry model: $p(u \prec v) = \sigma(\beta_{op}(z_v - z_u) + K_{uv})$, where $\sigma$ is the sigmoid function, $\beta_{op}$ is an inverse temperature parameter for the operation, and $K_{uv}$ captures baseline confusion.
    *   **Operation Accuracy:** For multi-choice or open-ended responses, we can use an IRT-like model: $P(\text{correct}|o, q) = \sigma(\alpha_o(\eta(q) - b_o - \delta_{\text{boundary}} \mathbf{1}[\text{boundary}] - \text{span_cost}(m) - \lambda s_v(t)))$, where $\eta(q)$ is item difficulty derived from structure (e.g., graph distance, length of segment $m$), $b_o$ is an overall operation bias, $\delta_{\text{boundary}}$ is a penalty for crossing a chunk boundary, $\text{span_cost}(m)$ models working memory load for tasks of length $m$, and $\lambda s_v(t)$ accounts for memory strength.
*   **Response Time (RT) Layer:** We model RT, which provides rich information about cognitive strategies.
    *   **Ex-Gaussian Model:** $RT \sim \mathrm{ExG}(\mu, \sigma, \tau)$, where $\mu$ is the mean of the normal component, $\sigma$ its standard deviation, and $\tau$ is the exponential decay rate. The $\mu$ parameter can be linked to latent state: $\mu = \lambda_0 + \lambda_1 \cdot \text{distance}(q) + \lambda_2 \cdot \mathbf{1}[\text{reverse}] + \lambda_3 \cdot \mathbf{1}[\text{boundary}] - \lambda_4 s_v(t) + u_{\text{subj}} + u_o$. Here, $\lambda_1$ captures the symbolic distance effect, $\lambda_2$ the cost of reversal, $\lambda_3$ the boundary crossing cost, and $\lambda_4$ the fluency benefit of strong memory.
    *   **Strategy Mixture (Optional):** More advanced models can hypothesize a mixture of cognitive strategies, e.g., $RT \sim \pi \cdot \mathrm{ExG}_{\text{serial_scan}} + (1-\pi) \cdot \mathrm{ExG}_{\text{direct_index}}$. A high $\pi$ would correspond to a strategy relying on mental traversal (high $\lambda_1$), while a low $\pi$ would indicate direct retrieval (low $\lambda_1$). Learning would entail a shift in $\pi$ from scan to index.

#### 3.4 Adaptive Item Selection

The core of adaptive training is to select the next query $q^*$ that will most effectively improve the learner's knowledge. Our policy aims to:

*   **Maximize Expected Information Gain (EIG):** Select $q^*$ that maximizes $E[\Delta \mathrm{Loss}(\text{learner's state}) | q^*]$, where loss combines accuracy, traversal symmetry penalties, and uncertainty over graph parameters ($z, B, \theta, K$). A cheaper surrogate is to select questions that reduce the uncertainty (entropy) of the latent graph parameters the most. For example, if the model has high uncertainty (wide posterior distribution) about the relative order of two nodes $u$ and $v$, it will prioritize a pairwise comparison between them.
*   **Target Difficulty:** Queries are filtered to be within an optimal learning zone (e.g., 70-80% success rate), balancing challenge and success.
*   **Coverage & Exploration:** A small $\epsilon$-greedy component ensures periodic sampling of less-practiced items or novel query types to prevent overfitting to the current model beliefs and to explore hidden weaknesses.
*   **Algorithmic Sketch:**
    1.  For each candidate query $q_i \in Q_{\text{candidate}}$ (where $Q_{\text{candidate}}$ is generated based on current learner state and task domain).
    2.  Calculate the expected information gain $U(q_i) = \mathbb{E}[KL(p(\theta | \mathcal{D}_{t}) || p(\theta | \mathcal{D}_{t}, \text{Response to } q_i))]$ where $\theta$ are the model parameters, and $KL$ is Kullback-Leibler divergence.
    3.  Select $q^* = \arg \max_{q_i} U(q_i)$, possibly with difficulty filtering or novelty bonus.
    4.  Present $q^*$, record response, update learner model, repeat.

This adaptive policy ensures that training is highly personalized, focusing resources where the learner's understanding of the graph structure is weakest or most uncertain.

### 4. Task Battery for Flexible Graph Traversal

To cultivate a graph-coded mental model, a diverse set of tasks is essential, each designed to probe and strengthen specific aspects of the underlying graph structure and its traversals. These tasks are automatically generated by the system based on the domain's graph.

#### 4.1 Core Operations for Ordered Sequences

For linear or ordered sequences (e.g., the English alphabet, or ordered lists of software commands), tasks target fluid bidirectional navigation and positional knowledge:

*   **Pairwise Order Queries:** "Which command comes earlier in this menu flow: `Save As` or `Export`?" (tests $z_u$ vs $z_v$). A variation: "Is `Print` between `File` and `Edit`?" (tests $z_u < z_v < z_w$).
*   **Successor/Predecessor Prompts:** "What command comes *before* `Copy` in the `Edit` menu?" or "What menu item is *after* `New` in the `File` menu?" (tests bidirectional adjacency, strengthens $E$).
*   **K-Jump Navigation:** "Starting at the `View` menu, what is 3 items *after* it in the standard menu bar?" (forces mental traversal or direct indexing).
*   **Segment Recital (Forward or Reverse):** "Recite the 4 letters preceding P in reverse order" (e.g., O, N, M, L). This trains flexible segment traversal and targets high reversal costs. A "reverse-N treadmill" drill (repetitively asking for N items in reverse) helps build fluency.
*   **Boundary Bridging:** For sequences chunked in memory (e.g., alphabet sections or menu groupings), tasks intentionally span these boundaries. "List 5 commands in reverse, starting from `Open`, where `Open` is usually the first item of a new command group." This forces learners to *stitch chunks together* rather than resetting to a known start/end point.
*   **Missing Item Completion:** "Complete the sequence: `File`, `Edit`, __, `View`, `Window`." (Tests local order knowledge and specific adjacency).
*   **Index Mapping:** "What is the 5th item in the default `File` menu?" or "What position in the default `Edit` menu is `Paste`?" (promotes direct access to ordinal positions, not just relative order).

#### 4.2 Cyclic Structures

For cyclic orders (e.g., days of the week, months of the year, color wheels in a design program), tasks ensure proper handling of wraparound:

*   **Directional Comparison with Wraparound:** "In the clock cycle, does 11:00 AM come before 2:00 PM if moving backwards?" (tests understanding of modular arithmetic).
*   **Successor/Predecessor with Wraparound:** "What month comes *after* December?" (tests circular link from end to beginning).
*   **Shortest Distance in Cycles:** "How many hours apart are 3:00 PM and 10:00 AM on a clock?" (requires finding minimum of two paths around the cycle).

#### 4.3 Partial Orders and DAGs

For domains with prerequisites or parallelizable steps (e.g., assembly instructions, software feature dependencies), tasks focus on understanding constraints and flexible sequencing:

*   **Comparability Queries:** "To use feature X, must feature Y be configured first, or can they be configured independently?" (tests dependency, explicitly probes incomparability).
*   **Path/Ancestor Queries:** "Is `Database Setup` a prerequisite for `User Authentication`?" (tests transitive closure).
*   **Topological Sort (Linear Extension):** Given a set of steps with dependencies, "Arrange tasks [A, B, C, D] in a valid order, where A before C, B before D, and C before D." (Learner must construct *any* valid linear extension, not just recall one).
*   **Minimal/Maximal Element Identification:** "Which setup steps have no prerequisites?" (Tests understanding of entry points in a DAG).
*   **Insertion and Adaptation:** "If a new `Security Audit` step must happen after `User Account Creation` but before `Data Encryption`, how does this change the workflow?" (Tests dynamic model adjustment).

#### 4.4 General Graph Navigation Tasks

For complex networks (e.g., public transport maps, large software menu trees), tasks foster planning and efficient navigation:

*   **Shortest Path Finding:** "What's the shortest route (sequence of clicks) from the `Welcome Screen` to `Printer Settings`?" (Exercises internal pathfinding).
*   **Next-Step Prediction:** "You are at `Dashboard`. To reach `User Profiles`, should you click `Reports` or `Settings` next?" (Tests local planning towards a goal).
*   **Landmark-based Navigation:** "To get from `Home Screen` to `Advanced Network Settings`, is it useful to go via the `Control Panel`?" (Fosters hierarchical chunking).
*   **Macro Discovery and Use:** Identifying and applying common sub-sequences (e.g., a "Login Macro" always involving `Click Login -> Enter Username -> Enter Password -> Click Submit`).

#### 4.5 Multi-Relation and Transfer Tasks

These tasks build higher-level cognitive flexibility:

*   **Filtering and Combined Criteria:** "What is the next `administrative` command after `Create User` in this sequence?" (Combines sequence order with semantic attribute).
*   **Projection Switching:** "If the `Format Disk` command is the 7th item in the `Tools` menu, what is its position when all commands are sorted by category?" (Forces mental switch between different structural views).
*   **Isomorphic Transfer:** After mastering one domain (e.g., software menu navigation for App A), introduce a new isomorphic domain (e.g., App B, with different labels but similar menu hierarchy) and measure learning rate on similar tasks. This gauges transfer of *structural learning strategies* rather than just content.

### 5. Metrics and Feedback for Graph-Coded Mastery

The detailed student model allows for objective, precise metrics and actionable feedback, moving beyond simple accuracy scores. These metrics operationalize "graph-coded mastery":

*   **Bidirectionality Index:** $\Delta RT_{\text{rev-fwd}}$ (response time for reverse task minus forward task of matched length and difficulty). Target: $\Delta RT \to 0$. Measures symmetry of access.
*   **Symbolic Distance Slope:** The slope of RT vs. graph distance for pairwise comparisons. Target: slope $\to 0$ (indicating direct retrieval rather than serial scanning).
*   **Wrap Penalty:** Increased RT or error rate for tasks requiring wraparound in cyclic orders. Target: penalty $\to 0$.
*   **Poset Fidelity:** Proportion of correct *incomparability* judgments ("these two steps can be done in either order"). Also, entropy over generated linear extensions (diverse answers if valid) for topological sort tasks.
*   **Locality of Errors:** Probability mass of errors within graph distance 1-2. We expect this to shrink and concentrate appropriately (e.g., early: many swaps; late: rare errors concentrated at remaining weak links or true boundaries).
*   **Strategy Shift Index ($\pi_t$):** For RT models with mixture components (e.g., serial scan vs. direct index), track the shift in $\pi_t$ over training. For example, a decrease in $\lambda_1$ (slope of RT vs. distance) means the learner is moving from a laborious, step-by-step scanning strategy to a more direct, indexed retrieval.
*   **Transfer Index:** Learning rate on a novel isomorphic domain normalized by baseline (e.g., compared to a control group without prior training).
*   **Chunk Boundary Penalty ($\delta_{\text{boundary}}$):** The estimated increase in RT or error probability when crossing inferred chunk boundaries. Target: $\delta_{\text{boundary}} \to 0$ (indicating "seam smoothing").

Feedback for the learner (and researchers) can be visualized:
*   **Uncertainty Map:** A heat map of confidence over the graph, showing blurry regions (high uncertainty in $z_v$) where knowledge is weak.
*   **Error Heatmap:** A visual $K_{ij}$ matrix showing which pairs of nodes are frequently confused.
*   **Performance Curves:** Plots of RT vs. distance, showing the slope flattening over training; reversal cost curves diminishing.
*   **Progress Report:** Explicitly showing improvement in metrics like "Your bidirectionality index is now 0.1s (down from 1.5s!)" or "You now correctly identify incomparable tasks 90% of the time."

### 6. Empirical Validation: A Behavioral Experiment

To validate the adaptive training system, we propose a controlled behavioral experiment. The primary focus will be on demonstrating the specific benefits of building graph-coded mental models on behavioral performance metrics.

#### 6.1 Experimental Design

*   **Participants:** Recruit a sufficient sample size (e.g., N=60-90, subject to power analysis) of participants with no prior expertise in the target domain.
*   **Domain Choice:** To control for prior knowledge, we will use a **synthetic alphabet** (e.g., 26 novel, visually distinct glyphs in a pre-defined linear order) or a **simple artificial partial order** (e.g., a 7-node DAG with specific constraints, unknown to participants). This ensures that any observed effects are due to our training, not pre-existing schemas. The "alphabet" example will serve as a simplified, clear context for initial piloting and illustration.
*   **Groups (Between-Subjects):**
    1.  **Adaptive-GCM Group:** Uses the full adaptive system, receiving tasks chosen to optimize the graph-coded mental model (maximizing EIG over $z, \theta, K, B$).
    2.  **Non-Adaptive Linear Group:** Receives training on the *same content* and *same total exposure time*, but only on a single, fixed linear traversal (e.g., always forward recall, simple flashcards of item A then B, then C, etc.). Task selection is non-adaptive, focusing on simple item repetition. This group serves as a control for "mere practice" and shows the limitations of linearization.
    3.  **Yoked Adaptive Control Group:** Participants receive a *yoked schedule* from a random participant in the Adaptive-GCM group. They get the *exact same sequence of tasks* and *task content* as their yoked partner, but their *individual performance is not used* to adapt the next question. This isolates the effect of adaptivity vs. merely being exposed to the right *types* of questions.

*   **Procedure:** Participants will undergo daily training sessions (e.g., 30 minutes/day for 5-7 days). Pre- and post-training assessments will be administered, including a delayed retention test (e.g., 1 week after training).

#### 6.2 Hypotheses (Linked to Model)

Our hypotheses are specific and tied to the proposed graph-coded mental model:

1.  **H1 (Representational Shift):** The Adaptive-GCM group will show a significantly greater decrease in the symbolic distance slope ($\lambda_1 \to 0$) for RTs on pairwise comparison tasks compared to both control groups. This indicates a shift from serial scanning to a more direct, indexed retrieval strategy (a "flat latency curve").
2.  **H2 (Seam Smoothing):** The Adaptive-GCM group will exhibit a significantly larger decrease in the chunk boundary penalty ($\delta_{\text{boundary}}$) for both RT and accuracy, relative to the control groups. This suggests successful integration of previously disparate "chunks" into a cohesive mental model.
3.  **H3 (Operator Disentangling):** The Adaptive-GCM group will demonstrate greater proficiency gains in complex operations (e.g., k-jump, reverse segment tasks), as measured by $\theta_o$, and these gains will generalize across varying difficulties (e.g., for reverse-N, the accuracy and RT will improve more uniformly across N).
4.  **H4 (Transfer):** On a novel, isomorphic graph (e.g., another synthetic sequence or DAG with different labels), the Adaptive-GCM group will demonstrate significantly faster initial learning and higher transfer efficiency (e.g., better initial performance, faster reduction of $\lambda_1$ or $\delta_{\text{boundary}}$) compared to both control groups.

#### 6.3 Measurements

*   **Behavioral:** Accuracy, Response Time (RT), and specific error patterns (e.g., adjacency errors, boundary errors) for all task types.
*   **Model-Derived Metrics:** The primary endpoints will be the change in the proposed metrics of graph-coded mastery: Bidirectionality Index, Symbolic Distance Slope, Chunk Boundary Penalty, Poset Fidelity, and Transfer Index.

#### 6.4 Analysis Plan

*   **Statistical Modeling:** We will fit hierarchical Bayesian models (e.g., using Stan) to the response data (accuracy and RT), directly estimating the latent parameters ($z, \theta, K, B$) and the coefficients ($\lambda_1, \delta_{\text{boundary}}$) for each participant.
*   **Primary Outcome Analysis:** Comparison of posterior distributions of parameter changes (e.g., $\Delta\lambda_1$, $\Delta\delta_{\text{boundary}}$) across groups using Bayesian contrasts and effect sizes (e.g., Cohen's d, or probability of superiority).
*   **Model Checks:** Posterior predictive checks will be used to ensure the model adequately captures observed RT distributions, error profiles, and boundary effects.
*   **Power Analysis:** We will conduct a power analysis (e.g., via simulation) to determine the required sample size to detect a practically meaningful effect size for our primary hypotheses (e.g., $\Delta\lambda_1 \le -0.3$ indicating a clear shift from scanning to indexing).
*   **Reproducibility:** Task generators, anonymized behavioral logs, and analysis code will be made publicly available.

#### 6.5 Limitations and Future Work (for this paper)

This initial study will be **behavioral-focused**. While the ultimate vision includes multi-modal validation, for a first paper demonstrating core effects, adding fNIRS/eye-tracking/HRV risks diluting the main message and adding complexity. We will mention these as **future research directions** in the discussion, explicitly stating they would be subjects of dedicated follow-up papers with their own preregistered hypotheses and detailed methodological considerations (e.g., fNIRS preprocessing, pupil confounds, HRV sensitivity to breathing). This allows us to deliver a sharp, defensible core contribution.

### 7. Discussion

This paper introduces an adaptive training system designed to cultivate **graph-coded mental models** for structured knowledge. By moving beyond rigid, linear memorization, our approach enables learners to build flexible, navigable schemas that support arbitrary traversals and complex operations.

The proposed probabilistic learner model integrates insights from psychometrics (IRT, Bradley-Terry), cognitive theories of sequence learning (positional coding, chunking), and memory (forgetting curves). It allows for detailed, real-time estimation of a learner's internal representation, including latent item embeddings, operation proficiencies, and specific error patterns and chunk boundaries. This fine-grained understanding is then leveraged by an adaptive item selection policy that maximizes information gain about the structural parameters of the learner's knowledge.

Our diverse task battery is specifically designed to probe and strengthen the various facets of graph mastery – from bidirectional adjacency and k-jumps in linear sequences, to wraparound in cycles, to comparability and topological sorting in partial orders, and shortest path finding in general graphs. The behavioral metrics we propose operationalize the concept of "graph-coded mastery," allowing us to quantify improvements in flexibility (e.g., diminishing bidirectionality costs), representational shifts (e.g., collapsing symbolic distance effects), and structural integration (e.g., reduced chunk boundary penalties).

The planned behavioral experiment, using controlled synthetic domains and robust control groups, is designed to provide clear evidence for the efficacy of our adaptive, graph-centric training. We anticipate that participants in the adaptive group will demonstrate superior performance on non-canonical queries, greater skill transfer to isomorphic domains, and exhibit specific shifts in cognitive strategy (e.g., from serial scanning to direct retrieval) as revealed by our model-derived metrics. This would support the claim that deliberately training a graph-coded mental model leads to a deeper, more robust, and flexible understanding of structured information.

**When is Flexibility Worth it?** It's important to acknowledge that for many domains, collapsing a partial order to a fixed linear order is indeed adaptive and efficient. For instance, reciting the alphabet forward, or a standard emergency procedure, benefits from automatized linearization. Our approach is most valuable in contexts where:
1.  **The underlying structure is large or irregular:** Uniform coverage by rote memorization is infeasible (e.g., a complex software UI with hundreds of commands).
2.  **Flexibility and adaptability are critical:** The task environment is dynamic, requiring adaptation to changes (e.g., new shortcuts, changed prerequisites), or the user needs to achieve goals via multiple paths.
3.  **Creative problem-solving is required:** The user must synthesize novel sequences or generate diverse solutions, rather than just recalling a predefined one (e.g., planning an optimal travel route on a multi-modal transport network).

Our system provides a means to systematically develop this flexibility, offering a principled alternative to the common human tendency to over-linearize complex knowledge.

**Broader Vision:** Beyond this initial study, the proposed system forms the basis for a general "learning game engine" for any domain that can be represented as a graph. This includes learning foreign language syntax trees, complex musical scales, or even abstract conceptual maps in scientific domains. The core methodology – defining the graph, modeling learner state over it, and adaptively training traversal operations – is broadly applicable.

**Future Directions:** Future work includes:
*   **Neuroscience Validation (Dedicated Paper):** As mentioned, subsequent studies could incorporate fNIRS, eye-tracking, and HRV to probe the neural and physiological correlates of building graph-coded mental models. For instance, testing the hypothesis that the adaptive group shows reduced prefrontal cortex activation for complex tasks (neural efficiency) or less physiological stress (HRV) as they achieve mastery.
*   **Real-World Application:** Deploying the system in real-world contexts like software training or specific educational curricula to assess ecological validity and impact on user productivity/learning outcomes.
*   **More Complex Graph Types:** Extending the model to handle more nuanced graph properties (e.g., weighted edges, attribute-rich nodes, dynamic graphs).
*   **Multi-User and Collaborative Learning:** Exploring how multiple learners interacting with the system, or collaboratively building a shared graph model, could further enhance learning.

In conclusion, by blending statistical modeling with cognitive task design, we propose a novel and powerful approach to training for flexible, graph-coded knowledge. This research aims to fundamentally improve how people acquire and apply structured information, moving beyond rote memorization to enable deeper, more adaptable, and ultimately more effective skill acquisition.
